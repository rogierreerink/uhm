use std::pin::Pin;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{stream::Peekable, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::DbError;

#[trait_variant::make(Send)]
pub trait ListDb {
    async fn get_multiple(&mut self) -> Result<Vec<List>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<List>;
    async fn create_multiple(&mut self, items: Vec<ListCreate>) -> Result<Vec<List>>;
    async fn update_by_id(&mut self, id: &Uuid, item: ListUpdate) -> Result<List>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type List = ListTemplate<Query>;
pub type ListCreate = ListDataTemplate<Create>;
pub type ListUpdate = ListDataTemplate<Update>;
pub type ListReference = ListTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ListTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<ListDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ListDataTemplate<M: Modifier> {
    #[serde(skip_serializing_if = "M::skip_data")]
    pub name: M::Data<String>,
}

impl FromRow<'_, PgRow> for List {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: ListDataTemplate {
                name: row.get("name"),
            },
        })
    }
}

macro_rules! next_matches_first {
    ($stream:ident, $first:ident, $($column_name:expr),+) => {
        if let Some(Ok(next)) = $stream.as_mut().peek().await {
            $(Some(next.get::<Uuid, _>($column_name)) == $first.get($column_name)) && +
        } else {
            false
        }
    };
}

impl List {
    async fn collect_lists(
        stream: impl Stream<Item = Result<PgRow, sqlx::Error>>,
    ) -> Result<Vec<List>> {
        let mut stream = std::pin::pin!(stream.peekable());
        let mut items = Vec::new();
        loop {
            let next = match stream.as_mut().try_next().await? {
                Some(next) => next,
                None => return Ok(items),
            };

            items.push(Self::collect_list(&next, &mut stream).await?);
        }
    }

    async fn collect_list(
        first: &PgRow,
        _rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<List> {
        Ok(List {
            id: first.get("id"),
            ts_created: first.get("ts_created"),
            ts_updated: first.get("ts_updated"),
            data: ListDataTemplate {
                name: first.get("name"),
            },
        })
    }
}

pub struct ListDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> ListDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl ListDb for ListDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<List>> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            "
            SELECT
                lists.id,
                lists.ts_created,
                lists.ts_updated,
                lists.name

            FROM public.lists

            ORDER BY lists.name
            ",
        )
        .fetch(&mut *conn);

        List::collect_lists(stream).await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<List> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<ListCreate>) -> Result<Vec<List>> {
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

    async fn update_by_id(&mut self, id: &Uuid, item: ListUpdate) -> Result<List> {
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

        // Relying on cascaded delete regarding corresponding list items
        if sqlx::query(
            "
            DELETE FROM public.lists
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

impl ListDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<List>
    where
        E: PgExecutor<'c>,
    {
        let stream = sqlx::query(
            "
            
            SELECT
                lists.id,
                lists.ts_created,
                lists.ts_updated,
                lists.name

            FROM public.lists

            WHERE lists.id = $1
            ",
        )
        .bind(id)
        .fetch(executor);

        match List::collect_lists(stream).await?.pop() {
            Some(item) => Ok(item),
            None => Err((DbError::NotFound).into()),
        }
    }

    async fn create(tx: &mut PgTransaction<'_>, create: ListCreate) -> Result<List> {
        let item_id = Uuid::new_v4();
        let item: List = sqlx::query_as(
            "
            INSERT INTO public.lists (id, name)
            VALUES ($1, $2)
            RETURNING id, ts_created, ts_updated, name
            ",
        )
        .bind(item_id)
        .bind(create.name)
        .fetch_one(&mut **tx)
        .await?;

        Ok(item)
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        update: ListUpdate,
    ) -> Result<List> {
        let mut item = Self::get_by_id(&mut **tx, id).await?;

        if let Some(name) = update.name {
            item.data.name = name;
        }

        let row = sqlx::query(
            "
            UPDATE public.lists
            SET name = $2,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING ts_updated
            ",
        )
        .bind(id)
        .bind(item.data.name.clone())
        .fetch_one(&mut **tx)
        .await?;

        item.ts_updated = row.get("ts_updated");

        Ok(item)
    }
}
