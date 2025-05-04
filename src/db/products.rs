use std::pin::Pin;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{stream::Peekable, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::{
    list_items::{ListItemDataTemplate, ListItemReference},
    lists::{ListDataTemplate, ListReference},
    DbError,
};

#[trait_variant::make(Send)]
pub trait ProductDb {
    async fn get_multiple(&mut self, params: SearchParams) -> Result<Vec<Product>>;
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
    #[serde(skip_deserializing)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_item_references: Option<Vec<ListItemReference>>,
}

#[derive(Default, Debug, Deserialize)]
pub struct SearchParams {
    pub name: Option<String>,
}

impl FromRow<'_, PgRow> for Product {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: ProductDataTemplate {
                name: row.get("name"),
                list_item_references: None,
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

impl Product {
    async fn collect_products(
        stream: impl Stream<Item = Result<PgRow, sqlx::Error>>,
    ) -> Result<Vec<Product>> {
        let mut stream = std::pin::pin!(stream.peekable());
        let mut items = Vec::new();
        loop {
            let next = match stream.as_mut().try_next().await? {
                Some(next) => next,
                None => return Ok(items),
            };

            items.push(Self::collect_product(&next, &mut stream).await?);
        }
    }

    async fn collect_product(
        first: &PgRow,
        rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<Product> {
        Ok(Product {
            id: first.get("id"),
            ts_created: first.get("ts_created"),
            ts_updated: first.get("ts_updated"),
            data: ProductDataTemplate {
                name: first.get("name"),
                list_item_references: Some({
                    let mut items = Vec::new();

                    if first.get::<Option<Uuid>, _>("list_id").is_some() {
                        items.push(Self::collect_list_item_refs(first, rest).await?);
                    }

                    loop {
                        if !next_matches_first!(rest, first, "id") {
                            break items;
                        }

                        let next = match rest.try_next().await? {
                            Some(next) => next,
                            None => break items,
                        };

                        items.push(Self::collect_list_item_refs(&next, rest).await?);
                    }
                }),
            },
        })
    }

    async fn collect_list_item_refs(
        first: &PgRow,
        _rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<ListItemReference> {
        Ok(ListItemReference {
            id: first.get("list_item_id"),
            data: Some(ListItemDataTemplate {
                list_reference: Some(ListReference {
                    id: first.get("list_id"),
                    data: Some(ListDataTemplate {
                        name: first.get("list_name"),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
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
    async fn get_multiple(&mut self, params: SearchParams) -> Result<Vec<Product>> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            "
            SELECT DISTINCT
                products.id,
                products.ts_created,
                products.ts_updated,
                products.name,
                list_items.id AS list_item_id,
                lists.id AS list_id,
                lists.name AS list_name,

                similarity($1, products.name) AS match_score

            FROM public.products
                LEFT JOIN public.product_list_items
                    ON products.id = product_list_items.product_id
                LEFT JOIN public.list_items
                    ON product_list_items.id = list_items.product_list_item_id
                LEFT JOIN public.lists
                    ON list_items.list_id = lists.id

            ORDER BY
                match_score DESC,
                products.name,
                lists.name
            ",
        )
        .bind(params.name)
        .fetch(&mut *conn);

        Product::collect_products(stream).await
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
        let stream = sqlx::query(
            "
            SELECT DISTINCT
                products.id,
                products.ts_created,
                products.ts_updated,
                products.name,
                list_items.id AS list_item_id,
                lists.id AS list_id,
                lists.name AS list_name

            FROM public.products
                LEFT JOIN public.product_list_items
                    ON products.id = product_list_items.product_id
                LEFT JOIN public.list_items
                    ON product_list_items.id = list_items.product_list_item_id
                LEFT JOIN public.lists
                    ON list_items.list_id = lists.id

            WHERE products.id = $1
            ",
        )
        .bind(id)
        .fetch(executor);

        match Product::collect_products(stream).await?.pop() {
            Some(item) => Ok(item),
            None => Err((DbError::NotFound).into()),
        }
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        update: ProductUpdate,
    ) -> Result<Product> {
        let mut item = Self::get_by_id(&mut **tx, id).await?;

        if let Some(name) = update.name {
            item.data.name = name;
        }

        let row = sqlx::query(
            "
            UPDATE public.products
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
