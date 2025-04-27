use std::pin::Pin;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{stream::Peekable, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::{
    list_items::{
        ListItemDataTemplate, ListItemKindTemplate, ListItemReference,
        TemporaryListItemDataTemplate, TemporaryListItemTemplate,
    },
    products::{ProductDataTemplate, ProductReference},
    DbError,
};

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
    #[serde(flatten)]
    #[serde(skip_serializing_if = "M::skip_data")]
    pub item_refs: M::Data<ListItemReferences<Reference>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ListItemReferences<M: Modifier> {
    #[serde(skip_serializing_if = "M::skip_data")]
    pub items: M::Data<Vec<ListItemReference>>,
}

impl FromRow<'_, PgRow> for List {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: ListDataTemplate {
                name: row.get("name"),
                item_refs: ListItemReferences { items: None },
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
        summary: bool,
    ) -> Result<Vec<List>> {
        let mut stream = std::pin::pin!(stream.peekable());
        let mut items = Vec::new();
        loop {
            let next = match stream.as_mut().try_next().await? {
                Some(next) => next,
                None => return Ok(items),
            };

            items.push(Self::collect_list(&next, &mut stream, summary).await?);
        }
    }

    async fn collect_list(
        first: &PgRow,
        rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
        summary: bool,
    ) -> Result<List> {
        Ok(List {
            id: first.get("id"),
            ts_created: first.get("ts_created"),
            ts_updated: first.get("ts_updated"),
            data: ListDataTemplate {
                name: first.get("name"),
                item_refs: if summary {
                    ListItemReferences::<Reference> { items: None }
                } else {
                    ListItemReferences::<Reference> {
                        items: Some({
                            let mut items = vec![Self::collect_item(first, rest).await?];
                            loop {
                                if !next_matches_first!(rest, first, "id") {
                                    break items;
                                }

                                let next = match rest.try_next().await? {
                                    Some(next) => next,
                                    None => break items,
                                };

                                items.push(Self::collect_item(&next, rest).await?);
                            }
                        }),
                    }
                },
            },
        })
    }

    async fn collect_item(
        first: &PgRow,
        _rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<ListItemReference> {
        Ok(ListItemReference {
            id: first.get("item_id"),
            data: Some(ListItemDataTemplate {
                checked: Some(first.get("item_checked")),
                kind: Some({
                    if let Some(id) = first.get("product_list_item_id") {
                        ListItemKindTemplate::Product {
                            link_id: id,
                            product: Some(ProductReference {
                                id: first.get("product_id"),
                                data: Some(ProductDataTemplate {
                                    name: Some(first.get("product_name")),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                        }
                    } else if let Some(id) = first.get("temporary_list_item_id") {
                        ListItemKindTemplate::Temporary {
                            link_id: id,
                            temporary: Some(TemporaryListItemTemplate {
                                data: Some(TemporaryListItemDataTemplate {
                                    name: Some(first.get("temporary_list_item_name")),
                                }),
                            }),
                        }
                    } else {
                        panic!("unreachable!")
                    }
                }),
                ..Default::default()
            }),
            ..Default::default()
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

        List::collect_lists(stream, true).await
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

        // Relying on cascaded delete of corresponding list items
        // TODO: make sure that item types (products, temporary) are removed as well
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
                lists.name,
                list_items.id AS item_id,
                list_items.ts_created AS item_ts_created,
                list_items.ts_updated AS item_ts_updated,
                list_items.checked AS item_checked,
                product_list_items.id AS product_list_item_id,
                products.id AS product_id,
                products.name AS product_name,
                temporary_list_items.id AS temporary_list_item_id,
                temporary_list_items.name AS temporary_list_item_name

            FROM public.lists
                LEFT JOIN public.list_items
                    ON lists.id = list_items.list_id
                LEFT JOIN public.product_list_items
                    ON list_items.product_list_item_id = product_list_items.id
                LEFT JOIN public.products
                    ON product_list_items.product_id = products.id
                LEFT JOIN public.temporary_list_items
                    ON list_items.temporary_list_item_id = temporary_list_items.id

            WHERE lists.id = $1
            ORDER BY
                lists.name,
                lists.id,
                COALESCE(products.name, temporary_list_items.name),
                list_items.id
            ",
        )
        .bind(id)
        .fetch(executor);

        match List::collect_lists(stream, false).await?.pop() {
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
