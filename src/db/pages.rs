use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::DbError;

#[trait_variant::make(Send)]
pub trait PageDb {
    async fn get_multiple(&mut self) -> Result<Vec<Page>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Page>;
    async fn create_multiple(&mut self, items: Vec<PageCreate>) -> Result<Vec<Page>>;
    async fn update_by_id(&mut self, id: &Uuid, item: PageUpdate) -> Result<Page>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type Page = PageTemplate<Query>;
pub type PageCreate = PageDataTemplate<Create>;
pub type PageUpdate = PageDataTemplate<Update>;
pub type PageReference = PageTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PageTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<PageDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PageDataTemplate<M: Modifier> {
    #[serde(skip_serializing_if = "M::skip_data")]
    pub name: M::Data<String>,
}

impl FromRow<'_, PgRow> for Page {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: PageDataTemplate::<Query> {
                name: row.get("name"),
            },
        })
    }
}

pub struct PageDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> PageDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl PageDb for PageDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<Page>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, name
            FROM public.pages
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Page> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<PageCreate>) -> Result<Vec<Page>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match sqlx::query_as(
                "
                INSERT INTO public.pages (id, name)
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

    async fn update_by_id(&mut self, id: &Uuid, item: PageUpdate) -> Result<Page> {
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
            DELETE FROM public.pages
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

impl PageDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<Page>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, name
            FROM public.pages
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
        update: PageUpdate,
    ) -> Result<Page> {
        let mut item = Self::get_by_id(&mut **tx, id).await?;

        if let Some(name) = update.name {
            item.data.name = name;
        }

        sqlx::query(
            "
            UPDATE public.pages
            SET name = $2,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING id, ts_created, ts_updated, name
            ",
        )
        .bind(id)
        .bind(item.data.name.clone())
        .execute(&mut **tx)
        .await?;

        Ok(item)
    }
}
