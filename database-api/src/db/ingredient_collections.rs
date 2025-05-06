use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{Stream, TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::{
    db::{
        ingredients::IngredientDataTemplate,
        products::{ProductDataTemplate, ProductReference},
    },
    utilities::modifier::{Create, Modifier, Query, Reference, Update},
};

use super::{ingredients::IngredientReference, DbError};

#[trait_variant::make(Send)]
pub trait IngredientCollectionDb {
    async fn get_multiple(&mut self) -> Result<Vec<IngredientCollection>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<IngredientCollection>;
    async fn create_multiple(
        &mut self,
        items: Vec<IngredientCollectionCreate>,
    ) -> Result<Vec<IngredientCollection>>;
    async fn update_by_id(
        &mut self,
        id: &Uuid,
        item: IngredientCollectionUpdate,
    ) -> Result<IngredientCollection>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type IngredientCollection = IngredientCollectionTemplate<Query>;
pub type IngredientCollectionCreate = IngredientCollectionDataTemplate<Create>;
pub type IngredientCollectionUpdate = IngredientCollectionDataTemplate<Update>;
pub type IngredientCollectionReference = IngredientCollectionTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IngredientCollectionTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<IngredientCollectionDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IngredientCollectionDataTemplate<M: Modifier> {
    #[serde(default)]
    #[serde(skip_serializing_if = "M::skip_data")]
    pub ingredients: M::Data<Vec<IngredientReference>>,
}

impl FromRow<'_, PgRow> for IngredientCollection {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: IngredientCollectionDataTemplate {
                ingredients: Vec::new(),
            },
        })
    }
}

impl FromRow<'_, PgRow> for IngredientReference {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(IngredientReference {
            id: row.get("ingredient_id"),
            data: Some(IngredientDataTemplate {
                product: Some(ProductReference {
                    id: row.get("product_id"),
                    data: Some(ProductDataTemplate {
                        name: Some(row.get("product_name")),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

impl IngredientCollection {
    async fn try_item_from_stream(
        rows: &mut (impl Stream<Item = Result<PgRow, sqlx::Error>> + Unpin),
    ) -> Result<Option<Self>> {
        let mut item = Option::None;

        while let Some(row) = rows.try_next().await? {
            let item = item.get_or_insert(IngredientCollection::from_row(&row)?);

            if let Some(_) = row.get::<Option<Uuid>, _>("ingredient_id") {
                item.data
                    .ingredients
                    .push(IngredientReference::from_row(&row)?);
            }
        }

        Ok(item)
    }

    async fn try_items_from_stream(
        rows: &mut (impl Stream<Item = Result<PgRow, sqlx::Error>> + Unpin),
    ) -> Result<Vec<Self>> {
        let mut items = HashMap::<Uuid, _>::new();

        while let Some(row) = rows.try_next().await? {
            let item = items
                .entry(row.get("id"))
                .or_insert(IngredientCollection::from_row(&row)?);

            if let Some(_) = row.get::<Option<Uuid>, _>("ingredient_id") {
                item.data
                    .ingredients
                    .push(IngredientReference::from_row(&row)?);
            }
        }

        Ok(items.into_values().collect())
    }
}

pub struct IngredientCollectionDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> IngredientCollectionDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl IngredientCollectionDb for IngredientCollectionDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<IngredientCollection>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            "
            SELECT
                ingredient_collections.id,
                ingredient_collections.ts_created,
                ingredient_collections.ts_updated,
                ingredients.id AS ingredient_id,
                products.id AS product_id,
                products.name AS product_name

            FROM public.ingredient_collections
                LEFT JOIN public.ingredients
                    ON ingredient_collections.id = ingredients.ingredient_collection_id
                LEFT JOIN public.products
                    ON ingredients.product_id = products.id

            ORDER BY ingredient_collections.id
            ",
        )
        .fetch(&mut *conn);

        IngredientCollection::try_items_from_stream(&mut stream).await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<IngredientCollection> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            "
            SELECT
                ingredient_collections.id,
                ingredient_collections.ts_created,
                ingredient_collections.ts_updated,
                ingredients.id AS ingredient_id,
                products.id AS product_id,
                products.name AS product_name

            FROM public.ingredient_collections
                LEFT JOIN public.ingredients
                    ON ingredient_collections.id = ingredients.ingredient_collection_id
                LEFT JOIN public.products
                    ON ingredients.product_id = products.id

            WHERE ingredient_collections.id = $1
            ",
        )
        .bind(id)
        .fetch(&mut *conn);

        match IngredientCollection::try_item_from_stream(&mut stream).await? {
            Some(item) => Ok(item),
            None => Err((DbError::NotFound).into()),
        }
    }

    async fn create_multiple(
        &mut self,
        items: Vec<IngredientCollectionCreate>,
    ) -> Result<Vec<IngredientCollection>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match Self::create(&mut tx, item).await {
                Ok(item) => created.push(item),
                Err(error) => {
                    tx.commit().await?;
                    return Err(error.into());
                }
            };
        }

        tx.commit().await?;

        Ok(created)
    }

    async fn update_by_id(
        &mut self,
        id: &Uuid,
        item: IngredientCollectionUpdate,
    ) -> Result<IngredientCollection> {
        let mut tx = self.pool.begin().await?;

        let updated = match Self::update_by_id(&mut tx, id, item).await {
            Ok(item) => item,
            Err(error) => {
                tx.commit().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(updated)
    }

    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        if sqlx::query(
            "
            DELETE FROM public.ingredient_collections
            WHERE id = $1
            ",
        )
        .bind(id)
        .execute(&mut *conn)
        .await?
        .rows_affected()
            == 0
        {
            return Err((DbError::NotFound).into());
        }

        Ok(())
    }
}

impl IngredientCollectionDbPostgres<'_> {
    async fn create(
        tx: &mut PgTransaction<'_>,
        _create: IngredientCollectionCreate,
    ) -> Result<IngredientCollection> {
        let item_id = Uuid::new_v4();
        let item = sqlx::query_as(
            "
            INSERT INTO public.ingredient_collections (id)
            VALUES ($1)
            RETURNING id, ts_created, ts_updated
            ",
        )
        .bind(item_id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(item)
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        _update: IngredientCollectionUpdate,
    ) -> Result<IngredientCollection> {
        let updated = sqlx::query_as(
            "
            UPDATE public.ingredient_collections
            SET ts_updated = NOW()
            WHERE id = $1
            RETURNING id, ts_created, ts_updated
            ",
        )
        .bind(id)
        .fetch_one(&mut **tx)
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => Into::<anyhow::Error>::into(DbError::NotFound),
            _ => error.into(),
        })
        .await?;

        Ok(updated)
    }
}
