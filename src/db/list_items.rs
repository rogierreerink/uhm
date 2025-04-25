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
pub trait ListItemDb {
    async fn get_multiple(&mut self) -> Result<Vec<ListItem>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<ListItem>;
    async fn create_multiple(&mut self, items: Vec<ListItemCreate>) -> Result<Vec<ListItem>>;
    async fn update_by_id(&mut self, id: &Uuid, item: ListItemUpdate) -> Result<ListItem>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type ListItem = ListItemTemplate<Query>;
pub type ListItemCreate = ListItemDataTemplate<Create>;
pub type ListItemUpdate = ListItemDataTemplate<Update>;
pub type ListItemReference = ListItemTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ListItemTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<ListItemDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ListItemDataTemplate<M: Modifier> {
    pub checked: M::Data<bool>,
    pub kind: M::Data<ListItemKindTemplate<M>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListItemKindTemplate<M: Modifier> {
    Product {
        #[serde(skip)]
        link_id: M::Meta<Uuid>,
        #[serde(flatten)]
        product: M::Data<ProductReference>,
    },
    Temporary {
        #[serde(skip)]
        link_id: M::Meta<Uuid>,
        #[serde(flatten)]
        temporary: M::Data<TemporaryListItemTemplate<M>>,
    },
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TemporaryListItemTemplate<M: Modifier> {
    pub data: M::Data<TemporaryListItemDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct TemporaryListItemDataTemplate<M: Modifier> {
    pub name: M::Data<String>,
}

impl FromRow<'_, PgRow> for ListItem {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: ListItemDataTemplate {
                checked: row.get("checked"),
                kind: {
                    if let Some(id) = row.get("product_list_item_id") {
                        ListItemKindTemplate::Product {
                            link_id: id,
                            product: ProductReference {
                                id: row.get("product_id"),
                                data: Some(ProductDataTemplate {
                                    name: Some(row.get("product_name")),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        }
                    } else if let Some(id) = row.get("temporary_list_item_id") {
                        ListItemKindTemplate::Temporary {
                            link_id: id,
                            temporary: TemporaryListItemTemplate {
                                data: TemporaryListItemDataTemplate {
                                    name: row.get("temporary_list_item_name"),
                                },
                            },
                        }
                    } else {
                        panic!("unreachable!")
                    }
                },
            },
        })
    }
}

pub struct ListItemDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> ListItemDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl ListItemDb for ListItemDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<ListItem>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT
                list_items.id,
                list_items.checked,
                list_items.ts_created,
                list_items.ts_updated,
                product_list_items.id AS product_list_item_id,
                products.id AS product_id,
                products.name AS product_name,
                temporary_list_items.id AS temporary_list_item_id,
                temporary_list_items.name AS temporary_list_item_name

            FROM public.list_items
                LEFT JOIN public.product_list_items
                    ON list_items.product_list_item_id = product_list_items.id
                LEFT JOIN public.products
                    ON product_list_items.product_id = products.id
                LEFT JOIN public.temporary_list_items
                    ON list_items.temporary_list_item_id = temporary_list_items.id

            ORDER BY
                CASE
                    WHEN product_list_items.id IS NOT NULL THEN products.name
                    WHEN temporary_list_items.id IS NOT NULL THEN temporary_list_items.name
                END,
                list_items.id
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<ListItem> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<ListItemCreate>) -> Result<Vec<ListItem>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match Self::create(&mut tx, item).await {
                Ok(item) => created.push(item),
                Err(error) => {
                    tx.rollback().await?;
                    return Err(error.into());
                }
            }
        }

        tx.commit().await?;

        Ok(created)
    }

    async fn update_by_id(&mut self, id: &Uuid, item: ListItemUpdate) -> Result<ListItem> {
        let mut tx = self.pool.begin().await?;

        let updated = match Self::update_by_id(&mut tx, id, item).await {
            Ok(item) => item,
            Err(error) => {
                tx.rollback().await?;
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
            DELETE FROM public.list_items
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

impl ListItemDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<ListItem>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT
                list_items.id,
                list_items.checked,
                list_items.ts_created,
                list_items.ts_updated,
                product_list_items.id AS product_list_item_id,
                products.id AS product_id,
                products.name AS product_name,
                temporary_list_items.id AS temporary_list_item_id,
                temporary_list_items.name AS temporary_list_item_name

            FROM public.list_items
                LEFT JOIN public.product_list_items
                    ON list_items.product_list_item_id = product_list_items.id
                LEFT JOIN public.products
                    ON product_list_items.product_id = products.id
                LEFT JOIN public.temporary_list_items
                    ON list_items.temporary_list_item_id = temporary_list_items.id

            WHERE list_items.id = $1
            ORDER BY
                CASE
                    WHEN product_list_items.id IS NOT NULL THEN products.name
                    WHEN temporary_list_items.id IS NOT NULL THEN temporary_list_items.name
                END,
                list_items.id
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

    async fn create(tx: &mut PgTransaction<'_>, item: ListItemCreate) -> Result<ListItem> {
        match item.kind {
            ListItemKindTemplate::Product { product, .. } => {
                let product_list_item_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "
                    INSERT INTO public.product_list_items (id, product_id)
                    VALUES ($1, $2)
                    ",
                )
                .bind(product_list_item_id)
                .bind(product.id)
                .execute(&mut **tx)
                .await?;

                let list_item_id = Uuid::new_v4();
                let list_item = sqlx::query(
                    "
                    INSERT INTO public.list_items (id, checked, product_list_item_id)
                    VALUES ($1, $2, $3)
                    RETURNING id, ts_created, checked
                    ",
                )
                .bind(list_item_id)
                .bind(item.checked)
                .bind(product_list_item_id)
                .fetch_one(&mut **tx)
                .await?;

                Ok(ListItem {
                    id: list_item_id,
                    ts_created: list_item.get("ts_created"),
                    ts_updated: None,
                    data: ListItemDataTemplate {
                        checked: list_item.get("checked"),
                        kind: ListItemKindTemplate::Product {
                            link_id: product_list_item_id,
                            product: ProductReference {
                                id: product.id,
                                ..Default::default()
                            },
                        },
                    },
                })
            }

            ListItemKindTemplate::Temporary { temporary, .. } => {
                let temporary_list_item_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "
                    INSERT INTO public.temporary_list_items (id, name)
                    VALUES ($1, $2)
                    ",
                )
                .bind(temporary_list_item_id.clone())
                .bind(temporary.data.name.clone())
                .execute(&mut **tx)
                .await?;

                let item_id = Uuid::new_v4();
                let item = sqlx::query(
                    "
                    INSERT INTO public.list_items (id, checked, temporary_list_item_id)
                    VALUES ($1, $2, $3)
                    RETURNING id, ts_created, checked
                    ",
                )
                .bind(item_id)
                .bind(item.checked)
                .bind(temporary_list_item_id)
                .fetch_one(&mut **tx)
                .await?;

                Ok(ListItem {
                    id: item_id,
                    ts_created: item.get("ts_created"),
                    ts_updated: None,
                    data: ListItemDataTemplate {
                        checked: item.get("checked"),
                        kind: ListItemKindTemplate::Temporary {
                            link_id: temporary_list_item_id,
                            temporary: TemporaryListItemTemplate {
                                data: TemporaryListItemDataTemplate {
                                    name: temporary.data.name,
                                },
                            },
                        },
                    },
                })
            }
        }
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        item: ListItemUpdate,
    ) -> Result<ListItem> {
        // Starts out as the current item, but gets updated as we go. Reusing it saves us the
        // clutter of creating and passing around new temporary variables.
        let mut current = Self::get_by_id(&mut **tx, id).await?;

        match &mut current.data.kind {
            ListItemKindTemplate::Product {
                link_id,
                product: current,
            } => match item.kind {
                Some(ListItemKindTemplate::Product {
                    product: update, ..
                }) => {
                    if let Some(update) = update {
                        current.id = update.id
                    }

                    sqlx::query(
                        "
                         UPDATE public.product_list_items
                         SET product_id = $2,
                             ts_updated = NOW()
                         WHERE id = $1
                         ",
                    )
                    .bind(link_id.clone())
                    .bind(current.id)
                    .execute(&mut **tx)
                    .await?;
                }

                // List item type cannot be changed
                Some(_) => return Err((DbError::InvalidOperation).into()),

                // Nothing to update
                _ => {}
            },

            ListItemKindTemplate::Temporary {
                link_id,
                temporary: current,
            } => match item.kind {
                Some(ListItemKindTemplate::Temporary {
                    temporary: update, ..
                }) => {
                    if let Some(update) = update {
                        if let Some(data) = update.data {
                            if let Some(name) = data.name {
                                current.data.name = name;
                            }
                        }
                    }

                    sqlx::query(
                        "
                         UPDATE public.temporary_list_items
                         SET name = $2,
                             ts_updated = NOW()
                         WHERE id = $1
                         ",
                    )
                    .bind(link_id.clone())
                    .bind(current.data.name.clone())
                    .execute(&mut **tx)
                    .await?;
                }

                // List item type cannot be changed
                Some(_) => return Err((DbError::InvalidOperation).into()),

                // Nothing to update
                _ => {}
            },
        };

        sqlx::query(
            "
             UPDATE public.list_items
             SET ts_updated = NOW()
             WHERE id = $1
             ",
        )
        .bind(id)
        .execute(&mut **tx)
        .await?;

        // Get the new state from the database
        Ok(Self::get_by_id(&mut **tx, id).await?)
    }
}
