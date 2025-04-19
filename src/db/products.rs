use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use variants::variants;

use crate::utilities::{group::GroupIterExt, pack::Pack};

use super::DbError;

#[trait_variant::make(Send)]
pub trait DbProducts {
    async fn get(&mut self, params: &QueryParams) -> Result<Vec<Product>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Product>;
    async fn create(&mut self, products: &Vec<ProductDataNew>) -> Result<Vec<ProductMinimal>>;
    async fn update(&mut self, id: &Uuid, product: &ProductDataUpdate) -> Result<ProductMinimal>;
    async fn delete(&mut self, id: &Uuid) -> Result<()>;
}

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    pub name: Option<String>,
}

#[variants(Minimal)]
#[derive(Serialize)]
pub struct Product {
    #[variants(include(Minimal))]
    pub id: Uuid,

    #[variants(include(Minimal))]
    pub ts_created: DateTime<Utc>,

    #[variants(include(Minimal))]
    pub ts_updated: Option<DateTime<Utc>>,

    #[variants(include(Minimal), retype = "{t}{v}")]
    pub data: ProductData,
}

impl From<&Vec<Row>> for Pack<Vec<Product>> {
    fn from(rows: &Vec<Row>) -> Self {
        rows.into_iter()
            .group_map(
                |row| row.get::<_, Uuid>("id"),
                |mut group| {
                    let row = *group.peek().unwrap();
                    let group = group.collect::<Vec<&Row>>();

                    Product {
                        id: row.get("id"),
                        ts_created: row.get("ts_created"),
                        ts_updated: row.get("ts_updated"),
                        data: Pack::<Option<_>>::from(&group)
                            .unpack()
                            .expect("cannot fail as `group` will never be empty"),
                    }
                },
            )
            .collect()
    }
}

#[variants(New, Update, Minimal)]
#[derive(Serialize, Deserialize)]
pub struct ProductData {
    #[variants(include(New, Minimal))]
    #[variants(include(Update), retype = "Option<{t}>")]
    pub name: String,
    pub shopping_list_item_links: Vec<ShoppingListItemLink>,
}

impl From<&Vec<&Row>> for Pack<Option<ProductData>> {
    fn from(rows: &Vec<&Row>) -> Self {
        rows.get(0)
            .map(|row| ProductData {
                name: row.get("name"),
                shopping_list_item_links: Pack::from(rows).unpack(),
            })
            .into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShoppingListItemLink {
    id: Uuid,
}

impl From<&Vec<&Row>> for Pack<Vec<ShoppingListItemLink>> {
    fn from(rows: &Vec<&Row>) -> Self {
        rows.into_iter()
            .filter_map(|row| {
                row.get::<_, Option<Uuid>>("shopping_list_item_id")
                    .map(|id| ShoppingListItemLink { id })
            })
            .collect()
    }
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
    async fn get(&mut self, params: &QueryParams) -> Result<Vec<Product>> {
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
        let products = Pack::from(&self.connection.query(&stmt, &[&params.name]).await?).unpack();

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
                    shopping_list.id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let products: Vec<Product> =
            Pack::from(&self.connection.query(&stmt, &[id]).await?).unpack();

        match products {
            products if products.len() == 0 => Err(DbError::NotFound.into()),
            products if products.len() >= 2 => Err(DbError::TooMany.into()),
            mut products => Ok(products.pop().unwrap()),
        }
    }

    async fn create(&mut self, products: &Vec<ProductDataNew>) -> Result<Vec<ProductMinimal>> {
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

            inserted.push(ProductMinimal {
                id: product_id,
                ts_created: row.get("ts_created"),
                ts_updated: None,
                data: ProductDataMinimal {
                    name: product.name.clone(),
                },
            })
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(inserted)
    }

    async fn update(&mut self, id: &Uuid, product: &ProductDataUpdate) -> Result<ProductMinimal> {
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
                    shopping_list.id
                ",
            )
            .await?;

        tracing::debug!("get current: executing query");
        let current =
            match Pack::<Vec<Product>>::from(&transaction.query(&stmt, &[id]).await?).unpack() {
                products if products.len() == 0 => return Err(DbError::NotFound.into()),
                products if products.len() >= 2 => return Err(DbError::TooMany.into()),
                mut products => products.pop().unwrap(),
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
        let updated_row = match transaction
            .query(
                &stmt,
                &[id, product.name.as_ref().unwrap_or(&current.data.name)],
            )
            .await?
        {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(ProductMinimal {
            id: id.clone(),
            ts_created: current.ts_created.clone(),
            ts_updated: updated_row.get("ts_updated"),
            data: ProductDataMinimal {
                name: product.name.as_ref().unwrap_or(&current.data.name).clone(),
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
        match transaction.execute(&stmt, &[id]).await? {
            rows if rows == 0 => return Err(DbError::NotFound.into()),
            rows if rows >= 2 => return Err(DbError::TooMany.into()),
            _ => (),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(())
    }
}
