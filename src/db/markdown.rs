use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::DbError;

#[trait_variant::make(Send)]
pub trait MarkdownDb {
    async fn get_multiple(&mut self) -> Result<Vec<Markdown>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Markdown>;
    async fn create_multiple(&mut self, items: Vec<MarkdownCreate>) -> Result<Vec<Markdown>>;
    async fn update_by_id(&mut self, id: &Uuid, item: MarkdownUpdate) -> Result<Markdown>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type Markdown = MarkdownTemplate<Query>;
pub type MarkdownCreate = MarkdownDataTemplate<Create>;
pub type MarkdownUpdate = MarkdownDataTemplate<Update>;
pub type MarkdownReference = MarkdownTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MarkdownTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<MarkdownDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MarkdownDataTemplate<M: Modifier> {
    pub markdown: M::Data<String>,
}

impl FromRow<'_, PgRow> for Markdown {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: MarkdownDataTemplate::<Query> {
                markdown: row.get("markdown"),
            },
        })
    }
}

pub struct MarkdownDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> MarkdownDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl MarkdownDb for MarkdownDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<Markdown>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, markdown
            FROM public.markdown
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Markdown> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<MarkdownCreate>) -> Result<Vec<Markdown>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match sqlx::query_as(
                "
                INSERT INTO public.markdown (id, markdown)
                VALUES ($1, $2)
                RETURNING id, ts_created, ts_updated, markdown
                ",
            )
            .bind(Uuid::new_v4())
            .bind(item.markdown)
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

    async fn update_by_id(&mut self, id: &Uuid, item: MarkdownUpdate) -> Result<Markdown> {
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
            DELETE FROM public.markdown
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

impl MarkdownDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<Markdown>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, markdown
            FROM public.markdown
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
        update: MarkdownUpdate,
    ) -> Result<Markdown> {
        let mut item = Self::get_by_id(&mut **tx, id).await?;

        if let Some(markdown) = update.markdown {
            item.data.markdown = markdown;
        }

        sqlx::query(
            "
            UPDATE public.markdown
            SET markdown = $2,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING id, ts_created, ts_updated, markdown
            ",
        )
        .bind(id)
        .bind(item.data.markdown.clone())
        .execute(&mut **tx)
        .await?;

        Ok(item)
    }
}
