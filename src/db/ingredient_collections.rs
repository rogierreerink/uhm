use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::DbError;

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
    #[serde(skip)]
    pub _phantom_data: M::Data<()>,
}

impl FromRow<'_, PgRow> for IngredientCollection {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: IngredientCollectionDataTemplate::<Query> { _phantom_data: () },
        })
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

        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated
            FROM public.ingredient_collections
            ORDER BY id
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<IngredientCollection> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
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
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<IngredientCollection>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated
            FROM public.ingredient_collections
            WHERE id = $1
            ",
        )
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
        .await?;

        Ok(updated)
    }
}
