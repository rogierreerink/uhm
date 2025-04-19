use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use variants::variants;

use super::DbError;

#[trait_variant::make(Send)]
pub trait DbListItems {
    async fn get(&mut self) -> Result<Vec<ListItem>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<ListItem>;
    async fn create(&mut self, list_items: &Vec<ListItemDataNew>) -> Result<Vec<ListItemMinimal>>;
    async fn update(
        &mut self,
        id: &Uuid,
        list_item: &ListItemDataUpdate,
    ) -> Result<ListItemMinimal>;
    async fn delete(&mut self, id: &Uuid) -> Result<()>;
}

#[variants(Minimal)]
#[derive(Serialize)]
pub struct ListItem {
    #[variants(include(Minimal))]
    pub id: Uuid,

    #[variants(include(Minimal))]
    pub ts_created: DateTime<Utc>,

    #[variants(include(Minimal))]
    pub ts_updated: Option<DateTime<Utc>>,

    #[variants(include(Minimal), retype = "{t}{v}")]
    pub data: ListItemData,
}

impl From<&Row> for ListItem {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

#[variants(Minimal, New, Update)]
#[derive(Serialize, Deserialize)]
pub struct ListItemData {
    #[variants(include(Minimal))]
    #[variants(include(New, Update), retype = "Option<{t}>")]
    pub checked: bool,

    #[variants(include(Minimal), retype = "{t}{v}")]
    #[variants(include(New), retype = "{t}{v}")]
    #[variants(include(Update), retype = "Option<{t}{v}>")]
    pub kind: ListItemKind,
}

impl From<&Row> for ListItemData {
    fn from(row: &Row) -> Self {
        Self {
            checked: row.get("checked"),
            kind: row.into(),
        }
    }
}

#[variants(Minimal, New, Update)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListItemKind {
    Product {
        #[variants(include(Minimal, New))]
        #[variants(include(Update), retype = "Option<{t}>")]
        id: Uuid,
        data: ProductData,
    },
    Temporary {
        #[variants(include(Minimal, New), retype = "{t}{v}")]
        #[variants(include(Update), retype = "Option<{t}{v}>")]
        data: TemporaryData,
    },
}

#[variants(Minimal)]
impl From<&Row> for ListItemKind {
    fn from(row: &Row) -> Self {
        if let Some(_) = row.get::<_, Option<Uuid>>("product_list_item_id") {
            Self::Product {
                #[variants(include(Minimal))]
                id: row.get("product_id"),
                data: row.into(),
            }
        } else if let Some(_) = row.get::<_, Option<Uuid>>("temporary_list_item_id") {
            Self::Temporary {
                #[variants(include(Minimal))]
                data: row.into(),
            }
        } else {
            panic!()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProductData {
    name: String,
}

impl From<&Row> for ProductData {
    fn from(row: &Row) -> Self {
        Self {
            name: row.get("product_name"),
        }
    }
}

#[variants(Minimal, New, Update)]
#[derive(Serialize, Deserialize)]
pub struct TemporaryData {
    #[variants(include(Minimal, New))]
    #[variants(include(Update), retype = "Option<{t}>")]
    name: String,
}

#[variants(Minimal)]
impl From<&Row> for TemporaryData {
    fn from(row: &Row) -> Self {
        Self {
            #[variants(include(Minimal))]
            name: row.get("temporary_name"),
        }
    }
}

pub struct DbListItemsPostgres {
    connection: Object<Manager>,
}

impl DbListItemsPostgres {
    pub fn new(connection: Object<Manager>) -> Self {
        Self { connection }
    }
}

impl DbListItems for DbListItemsPostgres {
    async fn get(&mut self) -> Result<Vec<ListItem>> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT list_items.id,
                    list_items.checked,
                    list_items.ts_created,
                    list_items.ts_updated,
                    temporary_list_items.id AS temporary_list_item_id,
                    temporary_list_items.name AS temporary_name,
                    product_list_items.id AS product_list_item_id,
                    products.id AS product_id,
                    products.name AS product_name

                FROM public.list_items
                    LEFT JOIN public.temporary_list_items
                        ON list_items.temporary_list_item_id = temporary_list_items.id
                    LEFT JOIN public.product_list_items
                        ON list_items.product_list_item_id = product_list_items.id
                    LEFT JOIN public.products
                        ON product_list_items.product_id = products.id

                ORDER BY
                    CASE
                        WHEN temporary_list_items.id IS NOT NULL THEN temporary_list_items.name
                        WHEN product_list_items.id IS NOT NULL THEN products.name
                    END,
                    list_items.id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let list_items = self
            .connection
            .query(&stmt, &[])
            .await?
            .into_iter()
            .map(|row| (&row).into())
            .collect();

        Ok(list_items)
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<ListItem> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT list_items.id,
                    list_items.checked,
                    list_items.ts_created,
                    list_items.ts_updated,
                    temporary_list_items.id AS temporary_list_item_id,
                    temporary_list_items.name AS temporary_name,
                    product_list_items.id AS product_list_item_id,
                    products.id AS product_id,
                    products.name AS product_name

                FROM public.list_items
                    LEFT JOIN public.temporary_list_items
                        ON list_items.temporary_list_item_id = temporary_list_items.id
                    LEFT JOIN public.product_list_items
                        ON list_items.product_list_item_id = product_list_items.id
                    LEFT JOIN public.products
                        ON product_list_items.product_id = products.id

                WHERE list_items.id = $1
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let list_item = match self.connection.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            rows => (&rows[0]).into(),
        };

        Ok(list_item)
    }

    async fn create(&mut self, list_items: &Vec<ListItemDataNew>) -> Result<Vec<ListItemMinimal>> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        let mut inserted = Vec::new();
        for list_item in list_items {
            let list_item_id = Uuid::new_v4();
            let kind_id = Uuid::new_v4();

            inserted.push(match &list_item.kind {
                ListItemKindNew::Product { id } => {
                    tracing::debug!("create product list item: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.product_list_items (
                                id,
                                product_id
                            )
                            VALUES (
                                $1, $2
                            )
                            ",
                        )
                        .await?;

                    tracing::debug!("create product list item: executing query");
                    transaction.execute(&stmt, &[&kind_id, id]).await?;

                    tracing::debug!("create list item: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.list_items (
                                id,
                                checked,
                                product_list_item_id
                            )
                            VALUES (
                                $1, $2, $3
                            )
                            RETURNING
                                checked,
                                ts_created
                            ",
                        )
                        .await?;

                    tracing::debug!("create list item: executing query");
                    let row = transaction
                        .query_one(&stmt, &[&list_item_id, &list_item.checked, &kind_id])
                        .await?;

                    ListItemMinimal {
                        id: list_item_id,
                        ts_created: row.get("ts_created"),
                        ts_updated: None,
                        data: ListItemDataMinimal {
                            checked: row.get("checked"),
                            kind: ListItemKindMinimal::Product { id: *id },
                        },
                    }
                }
                ListItemKindNew::Temporary { data } => {
                    tracing::debug!("create temporary list item: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.temporary_list_items (
                                id,
                                name
                            )
                            VALUES (
                                $1, $2
                            )
                            ",
                        )
                        .await?;

                    tracing::debug!("create temporary list item: executing query");
                    transaction.execute(&stmt, &[&kind_id, &data.name]).await?;

                    tracing::debug!("create list item: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.list_items (
                                id,
                                checked,
                                temporary_list_item_id
                            )
                            VALUES (
                                $1, $2, $3
                            )
                            RETURNING
                                checked,
                                ts_created
                            ",
                        )
                        .await?;

                    tracing::debug!("create list item: executing query");
                    let row = transaction
                        .query_one(&stmt, &[&list_item_id, &list_item.checked, &kind_id])
                        .await?;

                    ListItemMinimal {
                        id: list_item_id,
                        ts_created: row.get("ts_created"),
                        ts_updated: None,
                        data: ListItemDataMinimal {
                            checked: row.get("checked"),
                            kind: ListItemKindMinimal::Temporary {
                                data: TemporaryDataMinimal {
                                    name: data.name.clone(),
                                },
                            },
                        },
                    }
                }
            })
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(inserted)
    }

    async fn update(
        &mut self,
        id: &Uuid,
        list_item: &ListItemDataUpdate,
    ) -> Result<ListItemMinimal> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("get current list item: preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                SELECT list_items.id,
                    list_items.checked,
                    list_items.ts_created,
                    list_items.ts_updated,
                    temporary_list_items.id AS temporary_list_item_id,
                    temporary_list_items.name AS temporary_name,
                    product_list_items.id AS product_list_item_id,
                    products.id AS product_id

                FROM public.list_items
                    LEFT JOIN public.temporary_list_items
                        ON list_items.temporary_list_item_id = temporary_list_items.id
                    LEFT JOIN public.product_list_items
                        ON list_items.product_list_item_id = product_list_items.id
                    LEFT JOIN public.products
                        ON product_list_items.product_id = products.id

                WHERE list_items.id = $1
                ",
            )
            .await?;

        tracing::debug!("get current list item: executing query");
        let current = match transaction.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        let updated_kind = match &list_item.kind {
            Some(ListItemKindUpdate::Product { id }) => {
                let list_item_id: Uuid = match current.get("product_list_item_id") {
                    Some(id) => id,
                    None => return Err(DbError::InvalidOperation.into()),
                };

                tracing::debug!("update product list item: executing query");
                let stmt = transaction
                    .prepare_cached(
                        "
                        UPDATE public.product_list_items
                        SET product_id = $2,
                            ts_updated = CURRENT_TIMESTAMP
                        WHERE id = $1
                        ",
                    )
                    .await?;

                let product_id = id.unwrap_or(current.get("product_id"));

                tracing::debug!("update product list item: executing query");
                match transaction
                    .execute(&stmt, &[&list_item_id, &product_id])
                    .await?
                {
                    count if count == 0 => return Err(DbError::NotFound.into()),
                    count if count >= 2 => return Err(DbError::TooMany.into()),
                    _ => (),
                }

                ListItemKindMinimal::Product { id: product_id }
            }

            Some(ListItemKindUpdate::Temporary { data }) => {
                let list_item_id: Uuid = match current.get("temporary_list_item_id") {
                    Some(id) => id,
                    None => return Err(DbError::InvalidOperation.into()),
                };

                tracing::debug!("update temporary list item: executing query");
                let stmt = transaction
                    .prepare_cached(
                        "
                        UPDATE public.temporary_list_items
                        SET name = $2,
                            ts_updated = CURRENT_TIMESTAMP
                        WHERE id = $1
                        ",
                    )
                    .await?;

                let name = data
                    .as_ref()
                    .and_then(|data| data.name.clone())
                    .unwrap_or(current.get("temporary_name"));

                tracing::debug!("update temporary list item: executing query");
                match transaction.execute(&stmt, &[&list_item_id, &name]).await? {
                    count if count == 0 => return Err(DbError::NotFound.into()),
                    count if count >= 2 => return Err(DbError::TooMany.into()),
                    _ => (),
                }

                ListItemKindMinimal::Temporary {
                    data: TemporaryDataMinimal { name },
                }
            }

            None => ListItemKindMinimal::from(&current),
        };

        tracing::debug!("update list item: executing query");
        let stmt = transaction
            .prepare_cached(
                "
                UPDATE public.list_items
                SET checked = $2,
                    ts_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                RETURNING ts_updated
                ",
            )
            .await?;

        let checked = list_item.checked.unwrap_or(current.get("checked"));

        tracing::debug!("update list item: executing query");
        let updated = match transaction.query(&stmt, &[id, &checked]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(ListItemMinimal {
            id: *id,
            ts_created: current.get("ts_created"),
            ts_updated: updated.get("ts_updated"),
            data: ListItemDataMinimal {
                checked,
                kind: updated_kind,
            },
        })
    }

    async fn delete(&mut self, id: &Uuid) -> Result<()> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("delete list item: preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                DELETE FROM public.list_items
                WHERE id = $1
                RETURNING
                    product_list_item_id,
                    temporary_list_item_id
                ",
            )
            .await?;

        tracing::debug!("delete list item: executing query");
        let list_item = match transaction.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        if let Some(list_item_id) = list_item.get::<_, Option<Uuid>>("product_list_item_id") {
            tracing::debug!("delete product list item: preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    DELETE FROM public.product_list_items
                    WHERE id = $1
                    ",
                )
                .await?;

            tracing::debug!("delete product list item: executing query");
            match transaction.execute(&stmt, &[&list_item_id]).await? {
                count if count == 0 => return Err(DbError::NotFound.into()),
                count if count >= 2 => return Err(DbError::TooMany.into()),
                _ => (),
            };
        }

        if let Some(list_item_id) = list_item.get::<_, Option<Uuid>>("temporary_list_item_id") {
            tracing::debug!("delete temporary list item: preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    DELETE FROM public.temporary_list_items
                    WHERE id = $1
                    ",
                )
                .await?;

            tracing::debug!("delete temporary list item: executing query");
            match transaction.execute(&stmt, &[&list_item_id]).await? {
                count if count == 0 => return Err(DbError::NotFound.into()),
                count if count >= 2 => return Err(DbError::TooMany.into()),
                _ => (),
            };
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(())
    }
}
