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
pub trait DbIngredientCollections {
    async fn get(&mut self) -> Result<Vec<IngredientCollectionSummary>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<IngredientCollection>;
    async fn create(
        &mut self,
        ingredient_collections: &Vec<IngredientCollectionNew>,
    ) -> Result<Vec<IngredientCollectionSummary>>;
    async fn update(
        &mut self,
        id: &Uuid,
        ingredient_collection: &IngredientCollectionUpdate,
    ) -> Result<IngredientCollectionSummary>;
    async fn delete(&mut self, id: &Uuid) -> Result<()>;
}

#[derive(Serialize)]
pub struct IngredientCollectionSummary {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: IngredientCollectionSummaryData,
}

impl From<&Row> for IngredientCollectionSummary {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

#[derive(Serialize)]
pub struct IngredientCollectionSummaryData {}

impl From<&Row> for IngredientCollectionSummaryData {
    fn from(_: &Row) -> Self {
        Self {}
    }
}

#[derive(Serialize, Clone)]
pub struct IngredientCollection {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: IngredientCollectionData,
}

impl From<&Row> for IngredientCollection {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

impl From<&Vec<Row>> for QueryResult<IngredientCollection> {
    fn from(rows: &Vec<Row>) -> Self {
        rows.into_iter()
            .group_map(
                |row| row.get::<_, Uuid>("id"),
                |group| {
                    group
                        .fold(None, |collection, row| {
                            let mut collection: IngredientCollection =
                                collection.unwrap_or_else(|| row.into());

                            if let Some(_) = row.get::<_, Option<Uuid>>("ingredient_id") {
                                collection.data.ingredients.push(row.into());
                            }

                            Some(collection)
                        })
                        .unwrap()
                },
            )
            .collect()
    }
}

#[derive(Serialize, Clone)]
pub struct IngredientCollectionData {
    ingredients: Vec<Ingredient>,
}

impl From<&Row> for IngredientCollectionData {
    fn from(_: &Row) -> Self {
        Self {
            ingredients: vec![],
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Ingredient {
    id: Uuid,
    data: IngredientData,
}

impl From<&Row> for Ingredient {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("ingredient_id"),
            data: row.into(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct IngredientData {
    product: Product,
}

impl From<&Row> for IngredientData {
    fn from(row: &Row) -> Self {
        Self {
            product: row.into(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Product {
    id: Uuid,
    data: ProductData,
}

impl From<&Row> for Product {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("product_id"),
            data: row.into(),
        }
    }
}

#[derive(Serialize, Clone)]
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

#[derive(Deserialize)]
pub struct IngredientCollectionNew {}

#[derive(Deserialize)]
pub struct IngredientCollectionUpdate {}

pub struct DbIngredientCollectionsPostgres {
    connection: Object<Manager>,
}

impl DbIngredientCollectionsPostgres {
    pub fn new(connection: Object<Manager>) -> Self {
        Self { connection }
    }
}

impl DbIngredientCollections for DbIngredientCollectionsPostgres {
    async fn get(&mut self) -> Result<Vec<IngredientCollectionSummary>> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT id, ts_created, ts_updated
                FROM public.ingredient_collections
                ORDER BY id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let collections = self
            .connection
            .query(&stmt, &[])
            .await?
            .into_iter()
            .map(|row| (&row).into())
            .collect();

        Ok(collections)
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<IngredientCollection> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT
                    ingredient_collections.id,
                    ingredient_collections.ts_created,
                    ingredient_collections.ts_updated,
                    ingredients.id AS ingredient_id,
                    products.id AS product_id,
                    products.name AS product_name

                FROM public.ingredient_collections
                    LEFT JOIN public.ingredients
                        ON ingredient_collections.id = ingredients.ingredient_collection_id
                    LEFT JOIN public.products
                        ON ingredients.product_id = products.id

                WHERE ingredient_collections.id = $1
                ORDER BY
                    ingredient_collections.id,
                    products.name,
                    ingredients.id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let collections =
            QueryResult::<IngredientCollection>::from(&self.connection.query(&stmt, &[&id]).await?)
                .inner();

        match collections {
            collections if collections.len() == 0 => Err(DbError::NotFound.into()),
            collections if collections.len() >= 2 => Err(DbError::TooMany.into()),
            collections => Ok(collections[0].clone()),
        }
    }

    async fn create(
        &mut self,
        ingredient_collections: &Vec<IngredientCollectionNew>,
    ) -> Result<Vec<IngredientCollectionSummary>> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        let mut inserted = Vec::new();

        for _ in ingredient_collections {
            let ingredient_collection_id = Uuid::new_v4();

            tracing::debug!("preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    INSERT INTO public.ingredient_collections (id)
                    VALUES ($1)
                    RETURNING ts_created
                    ",
                )
                .await?;

            tracing::debug!("executing query");
            let row = transaction
                .query_one(&stmt, &[&ingredient_collection_id])
                .await?;

            inserted.push(IngredientCollectionSummary {
                id: ingredient_collection_id,
                ts_created: row.get("ts_created"),
                ts_updated: None,
                data: IngredientCollectionSummaryData {},
            });
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(inserted)
    }

    async fn update(
        &mut self,
        id: &Uuid,
        _: &IngredientCollectionUpdate,
    ) -> Result<IngredientCollectionSummary> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("get current: preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                SELECT id, ts_created, ts_updated
                FROM public.ingredient_collections
                WHERE id = $1
                ",
            )
            .await?;

        tracing::debug!("get current: executing query");
        let current = match transaction.query(&stmt, &[&id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            rows => rows[0].clone(),
        };

        tracing::debug!("update: executing query");
        let stmt = transaction
            .prepare_cached(
                "
                UPDATE public.ingredient_collections
                SET ts_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                RETURNING ts_updated
                ",
            )
            .await?;

        tracing::debug!("update: executing query");
        let updated = transaction.query_one(&stmt, &[&id]).await?;

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(IngredientCollectionSummary {
            id: current.get("id"),
            ts_created: current.get("ts_created"),
            ts_updated: updated.get("ts_updated"),
            data: IngredientCollectionSummaryData {},
        })
    }

    async fn delete(&mut self, id: &Uuid) -> Result<()> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                DELETE FROM public.ingredient_collections
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
