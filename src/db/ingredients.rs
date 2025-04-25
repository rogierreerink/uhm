use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::{
    products::{ProductDataTemplate, ProductReference},
    DbError,
};

#[trait_variant::make(Send)]
pub trait IngredientDb {
    async fn get_multiple(&mut self, collection_id: &Uuid) -> Result<Vec<Ingredient>>;
    async fn get_by_id(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<Ingredient>;
    async fn create_multiple(
        &mut self,
        collection_id: &Uuid,
        items: Vec<IngredientCreate>,
    ) -> Result<Vec<Ingredient>>;
    async fn update_by_id(
        &mut self,
        collection_id: &Uuid,
        id: &Uuid,
        item: IngredientUpdate,
    ) -> Result<Ingredient>;
    async fn delete_by_id(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<()>;
}

pub type Ingredient = IngredientTemplate<Query>;
pub type IngredientCreate = IngredientDataTemplate<Create>;
pub type IngredientUpdate = IngredientDataTemplate<Update>;
pub type IngredientReference = IngredientTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IngredientTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<IngredientDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IngredientDataTemplate<M: Modifier> {
    #[serde(skip_serializing_if = "M::skip_data")]
    pub product: M::Data<ProductReference>,
}

impl FromRow<'_, PgRow> for Ingredient {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: IngredientDataTemplate::<Query> {
                product: ProductReference {
                    id: row.get("product_id"),
                    data: Some(ProductDataTemplate {
                        name: Some(row.get("product_name")),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
        })
    }
}

pub struct IngredientDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> IngredientDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl IngredientDb for IngredientDbPostgres<'_> {
    async fn get_multiple(&mut self, collection_id: &Uuid) -> Result<Vec<Ingredient>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT
                ingredients.id,
                ingredients.ingredient_collection_id,
                ingredients.product_id,
                ingredients.ts_created,
                ingredients.ts_updated,
                products.name AS product_name

            FROM public.ingredients
                LEFT JOIN public.products
                    ON ingredients.product_id = products.id

            WHERE ingredients.ingredient_collection_id = $1
            ORDER BY products.name, ingredients.id
            ",
        )
        .bind(collection_id)
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<Ingredient> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, collection_id, id).await
    }

    async fn create_multiple(
        &mut self,
        collection_id: &Uuid,
        items: Vec<IngredientCreate>,
    ) -> Result<Vec<Ingredient>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match Self::create(&mut tx, collection_id, item).await {
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
        collection_id: &Uuid,
        id: &Uuid,
        item: IngredientUpdate,
    ) -> Result<Ingredient> {
        let mut tx = self.pool.begin().await?;

        let updated = match Self::update_by_id(&mut tx, collection_id, id, item).await {
            Ok(item) => item,
            Err(error) => {
                tx.commit().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(updated)
    }

    async fn delete_by_id(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        if sqlx::query(
            "
            DELETE FROM public.ingredients
            WHERE ingredient_collection_id = $1 AND id = $2
            ",
        )
        .bind(collection_id)
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

impl IngredientDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, collection_id: &Uuid, id: &Uuid) -> Result<Ingredient>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT
                ingredients.id,
                ingredients.ingredient_collection_id,
                ingredients.product_id,
                ingredients.ts_created,
                ingredients.ts_updated,
                products.name AS product_name

            FROM public.ingredients
                LEFT JOIN public.products
                    ON ingredients.product_id = products.id

            WHERE ingredients.ingredient_collection_id = $1 AND ingredients.id = $2
            ORDER BY products.name
            ",
        )
        .bind(collection_id)
        .bind(id)
        .fetch_one(executor)
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => Into::<anyhow::Error>::into(DbError::NotFound),
            _ => error.into(),
        })
        .await
    }

    async fn create(
        tx: &mut PgTransaction<'_>,
        collection_id: &Uuid,
        create: IngredientCreate,
    ) -> Result<Ingredient> {
        let item_id = Uuid::new_v4();
        let item = sqlx::query(
            "
            INSERT INTO public.ingredients (id, ingredient_collection_id, product_id)
            VALUES ($1, $2, $3)
            RETURNING ts_created, product_id
            ",
        )
        .bind(item_id)
        .bind(collection_id)
        .bind(create.product.id)
        .fetch_one(&mut **tx)
        .await?;

        Ok(Ingredient {
            id: item_id,
            ts_created: item.get("ts_created"),
            ts_updated: None,
            data: IngredientDataTemplate {
                product: ProductReference {
                    id: create.product.id,
                    ..Default::default()
                },
            },
        })
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        collection_id: &Uuid,
        id: &Uuid,
        update: IngredientUpdate,
    ) -> Result<Ingredient> {
        let mut item = Self::get_by_id(&mut **tx, collection_id, id).await?;

        if let Some(product) = update.product {
            item.data.product.id = product.id;
        }

        let _ = sqlx::query(
            "
            UPDATE public.ingredients
            SET product_id = $3,
                ts_updated = NOW()
            WHERE ingredient_collection_id = $1 AND id = $2
            ",
        )
        .bind(collection_id)
        .bind(id)
        .bind(item.id)
        .execute(&mut **tx)
        .await?;

        // Product data might have been invalidated, just leave it out
        item.data.product.data = None;

        Ok(item)
    }
}
