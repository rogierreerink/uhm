use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::DbError;

#[trait_variant::make(Send)]
pub trait ProductDb {
    async fn get_multiple(&mut self) -> Result<Vec<Product>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Product>;
    async fn create_multiple(&mut self, items: Vec<ProductCreate>) -> Result<Vec<Product>>;
    async fn update_by_id(&mut self, id: &Uuid, item: ProductUpdate) -> Result<Product>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type Product = ProductTemplate<Query>;
pub type ProductCreate = ProductDataTemplate<Create>;
pub type ProductUpdate = ProductDataTemplate<Update>;
pub type ProductReference = ProductTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ProductTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<ProductDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ProductDataTemplate<M: Modifier> {
    #[serde(skip_serializing_if = "M::skip_data")]
    pub name: M::Data<String>,
}

impl FromRow<'_, PgRow> for Product {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: ProductDataTemplate::<Query> {
                name: row.get("name"),
            },
        })
    }
}

pub struct ProductDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> ProductDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl ProductDb for ProductDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<Product>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, name
            FROM public.products
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Product> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<ProductCreate>) -> Result<Vec<Product>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match sqlx::query_as(
                "
                INSERT INTO public.products (id, name)
                VALUES ($1, $2)
                RETURNING id, ts_created, ts_updated, name
                ",
            )
            .bind(Uuid::new_v4())
            .bind(item.name)
            .fetch_one(&mut *tx)
            .await
            {
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

    async fn update_by_id(&mut self, id: &Uuid, item: ProductUpdate) -> Result<Product> {
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
            DELETE FROM public.products
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

impl ProductDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<Product>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, name
            FROM public.products
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

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        item: ProductUpdate,
    ) -> Result<Product> {
        let current = Self::get_by_id(&mut **tx, id).await?;
        let updated = sqlx::query_as(
            "
            UPDATE public.products
            SET name = $2,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING id, ts_created, ts_updated, name
            ",
        )
        .bind(id)
        .bind(item.name.unwrap_or(current.data.name))
        .fetch_one(&mut **tx)
        .await?;

        Ok(updated)
    }
}
