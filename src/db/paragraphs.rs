use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::DbError;

#[trait_variant::make(Send)]
pub trait ParagraphDb {
    async fn get_multiple(&mut self) -> Result<Vec<Paragraph>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Paragraph>;
    async fn create_multiple(&mut self, items: Vec<ParagraphCreate>) -> Result<Vec<Paragraph>>;
    async fn update_by_id(&mut self, id: &Uuid, item: ParagraphUpdate) -> Result<Paragraph>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type Paragraph = ParagraphTemplate<Query>;
pub type ParagraphCreate = ParagraphDataTemplate<Create>;
pub type ParagraphUpdate = ParagraphDataTemplate<Update>;
pub type ParagraphReference = ParagraphTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ParagraphTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    pub ts_created: M::Meta<DateTime<Utc>>,
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    pub data: M::Data<ParagraphDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ParagraphDataTemplate<M: Modifier> {
    pub text: M::Data<String>,
}

impl FromRow<'_, PgRow> for Paragraph {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: ParagraphDataTemplate::<Query> {
                text: row.get("text"),
            },
        })
    }
}

pub struct ParagraphDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> ParagraphDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl ParagraphDb for ParagraphDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<Paragraph>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, text
            FROM public.paragraphs
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Paragraph> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<ParagraphCreate>) -> Result<Vec<Paragraph>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            created.push(
                match sqlx::query_as(
                    "
                    INSERT INTO public.paragraphs (id, text)
                    VALUES ($1, $2)
                    RETURNING id, ts_created, ts_updated, text
                    ",
                )
                .bind(Uuid::new_v4())
                .bind(item.text)
                .fetch_one(&mut *tx)
                .await
                {
                    Ok(created) => created,
                    Err(error) => {
                        tx.commit().await?;
                        return Err(error.into());
                    }
                },
            );
        }

        tx.commit().await?;

        Ok(created)
    }

    async fn update_by_id(&mut self, id: &Uuid, item: ParagraphUpdate) -> Result<Paragraph> {
        let mut tx = self.pool.begin().await?;

        let current = Self::get_by_id(&mut *tx, id).await?;
        let updated = sqlx::query_as(
            "
            UPDATE public.paragraphs
            SET text = $2,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING id, ts_created, ts_updated, text
            ",
        )
        .bind(id)
        .bind(item.text.unwrap_or(current.data.text))
        .fetch_one(&mut *tx)
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => Into::<anyhow::Error>::into(DbError::NotFound),
            _ => error.into(),
        })
        .await?;

        tx.commit().await?;

        Ok(updated)
    }

    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        if sqlx::query(
            "
            DELETE FROM public.paragraphs
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

impl ParagraphDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<Paragraph>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT id, ts_created, ts_updated, text
            FROM public.paragraphs
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
}
