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
    async fn get_multiple(&mut self, list_id: &Uuid) -> Result<Vec<ListItem>>;
    async fn get_by_id(&mut self, list_id: &Uuid, id: &Uuid) -> Result<ListItem>;
    async fn create_multiple(
        &mut self,
        list_id: &Uuid,
        items: Vec<ListItemCreate>,
    ) -> Result<Vec<ListItem>>;
    async fn update_by_id(
        &mut self,
        list_id: &Uuid,
        id: &Uuid,
        item: ListItemUpdate,
    ) -> Result<ListItem>;
    async fn delete_by_id(&mut self, list_id: &Uuid, id: &Uuid) -> Result<()>;
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
    #[serde(skip_serializing_if = "M::skip_data")]
    pub checked: M::Data<bool>,
    #[serde(skip_serializing_if = "M::skip_data")]
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
    async fn get_multiple(&mut self, list_id: &Uuid) -> Result<Vec<ListItem>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT
                list_items.id,
                list_items.ts_created,
                list_items.ts_updated,
                list_items.checked,
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

            WHERE list_items.list_id = $1
            ORDER BY
                COALESCE(products.name, temporary_list_items.name),
                list_items.id
            ",
        )
        .bind(list_id)
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, list_id: &Uuid, id: &Uuid) -> Result<ListItem> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, list_id, id).await
    }

    async fn create_multiple(
        &mut self,
        list_id: &Uuid,
        items: Vec<ListItemCreate>,
    ) -> Result<Vec<ListItem>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match Self::create(&mut tx, list_id, item).await {
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

    async fn update_by_id(
        &mut self,
        list_id: &Uuid,
        id: &Uuid,
        item: ListItemUpdate,
    ) -> Result<ListItem> {
        let mut tx = self.pool.begin().await?;

        let updated = match Self::update_by_id(&mut tx, list_id, id, item).await {
            Ok(item) => item,
            Err(error) => {
                tx.rollback().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(updated)
    }

    async fn delete_by_id(&mut self, list_id: &Uuid, id: &Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        match Self::delete_by_id(&mut tx, list_id, id).await {
            Ok(item) => item,
            Err(error) => {
                tx.rollback().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(())
    }
}

impl ListItemDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, list_id: &Uuid, id: &Uuid) -> Result<ListItem>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT
                list_items.id,
                list_items.ts_created,
                list_items.ts_updated,
                list_items.checked,
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

            WHERE
                list_items.list_id = $1 AND
                list_items.id = $2

            ORDER BY
                COALESCE(products.name, temporary_list_items.name),
                list_items.id
            ",
        )
        .bind(list_id)
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
        list_id: &Uuid,
        create: ListItemCreate,
    ) -> Result<ListItem> {
        match create.kind {
            ListItemKindTemplate::Product { product, .. } => {
                let link_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "
                    INSERT INTO public.product_list_items (id, product_id)
                    VALUES ($1, $2)
                    ",
                )
                .bind(link_id)
                .bind(product.id)
                .execute(&mut **tx)
                .await?;

                let item_id = Uuid::new_v4();
                let item = sqlx::query(
                    "
                    INSERT INTO public.list_items (id, list_id, checked, product_list_item_id)
                    VALUES ($1, $2, $3, $4)
                    RETURNING ts_created, checked
                    ",
                )
                .bind(item_id)
                .bind(list_id)
                .bind(create.checked)
                .bind(link_id)
                .fetch_one(&mut **tx)
                .await?;

                Ok(ListItem {
                    id: item_id,
                    ts_created: item.get("ts_created"),
                    ts_updated: None,
                    data: ListItemDataTemplate {
                        checked: item.get("checked"),
                        kind: ListItemKindTemplate::Product {
                            link_id,
                            product: ProductReference {
                                id: product.id,
                                ..Default::default()
                            },
                        },
                    },
                })
            }

            ListItemKindTemplate::Temporary { temporary, .. } => {
                let link_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "
                    INSERT INTO public.temporary_list_items (id, name)
                    VALUES ($1, $2)
                    ",
                )
                .bind(link_id.clone())
                .bind(temporary.data.name.clone())
                .execute(&mut **tx)
                .await?;

                let item_id = Uuid::new_v4();
                let item = sqlx::query(
                    "
                    INSERT INTO public.list_items (id, list_id, checked, temporary_list_item_id)
                    VALUES ($1, $2, $3, $4)
                    RETURNING ts_created, checked
                    ",
                )
                .bind(item_id)
                .bind(list_id)
                .bind(create.checked)
                .bind(link_id)
                .fetch_one(&mut **tx)
                .await?;

                Ok(ListItem {
                    id: item_id,
                    ts_created: item.get("ts_created"),
                    ts_updated: None,
                    data: ListItemDataTemplate {
                        checked: item.get("checked"),
                        kind: ListItemKindTemplate::Temporary {
                            link_id,
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
        list_id: &Uuid,
        id: &Uuid,
        update: ListItemUpdate,
    ) -> Result<ListItem> {
        let mut item = Self::get_by_id(&mut **tx, list_id, id).await?;

        match &mut item.data.kind {
            ListItemKindTemplate::Product {
                link_id,
                product: current,
            } => match update.kind {
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

                    // Data might have been invalidated, just leave it out
                    current.data = None;
                }

                // List item type cannot be changed
                Some(_) => return Err((DbError::InvalidOperation).into()),

                // Nothing to update
                _ => {}
            },

            ListItemKindTemplate::Temporary {
                link_id,
                temporary: current,
            } => match update.kind {
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

        let row = sqlx::query(
            "
             UPDATE public.list_items
             SET ts_updated = NOW()
             WHERE id = $1
             RETURNING ts_updated
             ",
        )
        .bind(id)
        .fetch_one(&mut **tx)
        .await?;

        item.ts_updated = row.get("ts_updated");

        Ok(item)
    }

    async fn delete_by_id(tx: &mut PgTransaction<'_>, list_id: &Uuid, id: &Uuid) -> Result<()> {
        let item = Self::get_by_id(&mut **tx, list_id, id).await?;

        let delete_link_query = match item.data.kind {
            ListItemKindTemplate::Product { link_id, .. } => sqlx::query(
                "
                DELETE FROM public.product_list_items
                WHERE id = $1
                ",
            )
            .bind(link_id)
            .execute(&mut **tx),

            ListItemKindTemplate::Temporary { link_id, .. } => sqlx::query(
                "
                DELETE FROM public.temporary_list_items
                WHERE id = $1
                ",
            )
            .bind(link_id)
            .execute(&mut **tx),
        };

        // We are relying on cascaded deletion of the main item
        if delete_link_query.await?.rows_affected() == 0 {
            return Err((DbError::InvalidContent).into());
        }

        Ok(())
    }
}
