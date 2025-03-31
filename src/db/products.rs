use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::utilities::group::GroupIterExt;

use super::{DbError, QueryResult};

#[trait_variant::make(Send)]
pub trait DbProducts {
    async fn get(&mut self, params: &QueryParams) -> Result<Vec<ProductSummary>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Product>;
    async fn create(&mut self, products: &Vec<ProductNew>) -> Result<Vec<Product>>;
    async fn update(&mut self, id: &Uuid, product: &ProductUpdate) -> Result<Product>;
    async fn delete(&mut self, id: &Uuid) -> Result<()>;
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct ProductSummary {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: ProductSummaryData,
}

impl From<&Row> for ProductSummary {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

impl From<&Vec<Row>> for QueryResult<ProductSummary> {
    fn from(rows: &Vec<Row>) -> Self {
        rows.into_iter()
            .group_map(
                |id| id.get::<_, Uuid>("id"),
                |group| {
                    group
                        .fold(None, |product, row| {
                            let mut product: ProductSummary = product.unwrap_or_else(|| row.into());

                            if let Some(_) = row.get::<_, Option<Uuid>>("shopping_list_item_id") {
                                product.data.shopping_list_item_links.push(row.into());
                            }

                            Some(product)
                        })
                        .unwrap()
                },
            )
            .collect()
    }
}

#[derive(Serialize)]
pub struct ProductSummaryData {
    pub name: String,
    pub shopping_list_item_links: Vec<ShoppingListItemLinkSummary>,
}

impl From<&Row> for ProductSummaryData {
    fn from(row: &Row) -> Self {
        Self {
            name: row.get("name"),
            shopping_list_item_links: vec![],
        }
    }
}

#[derive(Serialize)]
pub struct ShoppingListItemLinkSummary {
    id: Uuid,
}

impl From<&Row> for ShoppingListItemLinkSummary {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("shopping_list_item_id"),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Product {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: ProductData,
}

impl From<&Row> for Product {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

impl From<&Vec<Row>> for QueryResult<Product> {
    fn from(rows: &Vec<Row>) -> Self {
        rows.into_iter()
            .group_map(
                |row| row.get::<_, Uuid>("id"),
                |group| {
                    group
                        .fold(None, |product, row| {
                            let mut product: Product = product.unwrap_or_else(|| row.into());

                            if let Some(_) = row.get::<_, Option<Uuid>>("shopping_list_item_id") {
                                product.data.shopping_list_item_links.push(row.into());
                            }

                            Some(product)
                        })
                        .unwrap()
                },
            )
            .collect()
    }
}

#[derive(Serialize, Clone)]
pub struct ProductData {
    pub name: String,
    pub shopping_list_item_links: Vec<ShoppingListItemLink>,
}

impl From<&Row> for ProductData {
    fn from(row: &Row) -> Self {
        Self {
            name: row.get("name"),
            shopping_list_item_links: vec![],
        }
    }
}

#[derive(Serialize, Clone)]
pub struct ShoppingListItemLink {
    id: Uuid,
}

impl From<&Row> for ShoppingListItemLink {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("shopping_list_item_id"),
        }
    }
}

#[derive(Deserialize)]
pub struct ProductNew {
    pub name: String,
}

#[derive(Deserialize)]
pub struct ProductUpdate {
    pub name: Option<String>,
}

pub struct DbProductsPostgres {
    connection: Object<Manager>,
}

impl DbProductsPostgres {
    pub fn new(connection: Object<Manager>) -> Self {
        Self { connection }
    }
}

impl DbProducts for DbProductsPostgres {
    async fn get(&mut self, params: &QueryParams) -> Result<Vec<ProductSummary>> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT
                    products.id,
                    products.name,
                    products.ts_created,
                    products.ts_updated,
                    shopping_list.id AS shopping_list_item_id

                FROM public.products
                    LEFT JOIN public.shopping_list_product_links
                        ON products.id = shopping_list_product_links.product_id
                    LEFT JOIN public.shopping_list
                        ON shopping_list_product_links.id = shopping_list.product_link_id

                WHERE CAST($1 AS VARCHAR) IS NULL
                    OR (CAST($1 AS VARCHAR) IS NOT NULL AND name ~* $1)

                ORDER BY
                    products.name,
                    products.id,
                    shopping_list.id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let products =
            QueryResult::from(&self.connection.query(&stmt, &[&params.name]).await?).inner();

        Ok(products)
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Product> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT
                    products.id,
                    products.name,
                    products.ts_created,
                    products.ts_updated,
                    shopping_list.id AS shopping_list_item_id

                FROM public.products
                    LEFT JOIN public.shopping_list_product_links
                        ON products.id = shopping_list_product_links.product_id
                    LEFT JOIN public.shopping_list
                        ON shopping_list_product_links.id = shopping_list.product_link_id

                WHERE products.id = $1
                ORDER BY
                    products.name,
                    products.id,
                    shopping_list.id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let products =
            QueryResult::<Product>::from(&self.connection.query(&stmt, &[&id]).await?).inner();

        match products {
            products if products.len() == 0 => Err(DbError::NotFound.into()),
            products if products.len() >= 2 => Err(DbError::TooMany.into()),
            products => Ok(products[0].clone()),
        }
    }

    async fn create(&mut self, products: &Vec<ProductNew>) -> Result<Vec<Product>> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        let mut inserted = Vec::new();

        for product in products {
            let product_id = Uuid::new_v4();

            tracing::debug!("preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    INSERT INTO public.products (
                        id, 
                        name
                    )
                    VALUES (
                        $1, $2
                    )
                    RETURNING ts_created
                    ",
                )
                .await?;

            tracing::debug!("executing query");
            let row = transaction
                .query_one(&stmt, &[&product_id, &product.name])
                .await?;

            inserted.push(Product {
                id: product_id,
                ts_created: row.get("ts_created"),
                ts_updated: None,
                data: ProductData {
                    name: product.name.clone(),
                    shopping_list_item_links: vec![],
                },
            })
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(inserted)
    }

    async fn update(&mut self, id: &Uuid, product: &ProductUpdate) -> Result<Product> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("get current: preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                SELECT
                    products.id,
                    products.name,
                    products.ts_created,
                    products.ts_updated,
                    shopping_list.id AS shopping_list_item_id

                FROM public.products
                    LEFT JOIN public.shopping_list_product_links
                        ON products.id = shopping_list_product_links.product_id
                    LEFT JOIN public.shopping_list
                        ON shopping_list_product_links.id = shopping_list.product_link_id

                WHERE products.id = $1
                ORDER BY
                    products.name,
                    products.id,
                    shopping_list.id
                ",
            )
            .await?;

        tracing::debug!("get current: executing query");
        let current =
            match QueryResult::<Product>::from(&transaction.query(&stmt, &[&id]).await?).inner() {
                products if products.len() == 0 => return Err(DbError::NotFound.into()),
                products if products.len() >= 2 => return Err(DbError::TooMany.into()),
                products => products[0].clone(),
            };

        tracing::debug!("update: executing query");
        let stmt = transaction
            .prepare_cached(
                "
                UPDATE public.products
                SET name = $2,
                    ts_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                RETURNING ts_updated
                ",
            )
            .await?;

        tracing::debug!("update: executing query");
        let updated_row = transaction
            .query_one(
                &stmt,
                &[&id, product.name.as_ref().unwrap_or(&current.data.name)],
            )
            .await?;

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(Product {
            id: id.clone(),
            ts_created: current.ts_created.clone(),
            ts_updated: updated_row.get("ts_updated"),
            data: ProductData {
                name: product.name.as_ref().unwrap_or(&current.data.name).clone(),
                shopping_list_item_links: current.data.shopping_list_item_links,
            },
        })
    }

    async fn delete(&mut self, id: &Uuid) -> Result<()> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                DELETE FROM public.products
                WHERE id = $1
                ",
            )
            .await?;

        tracing::debug!("executing query");
        match transaction.execute(&stmt, &[&id]).await? {
            rows if rows == 0 => return Err(DbError::NotFound.into()),
            rows if rows >= 2 => return Err(DbError::TooMany.into()),
            _ => (),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(())
    }
}
