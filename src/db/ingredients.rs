use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use super::DbError;

#[trait_variant::make(Send)]
pub trait DbIngredients {
    async fn get(&mut self, collection_id: &Uuid) -> Result<Vec<IngredientSummary>>;
    async fn get_by_id(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<Ingredient>;
    async fn create(
        &mut self,
        collection_id: &Uuid,
        ingredients: &Vec<IngredientNew>,
    ) -> Result<Vec<IngredientMinimal>>;
    async fn update(
        &mut self,
        id: &Uuid,
        collection_id: &Uuid,
        ingredient: &IngredientUpdate,
    ) -> Result<IngredientMinimal>;
    async fn delete(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<()>;
}

#[derive(Serialize)]
pub struct IngredientSummary {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: IngredientSummaryData,
}

impl From<&Row> for IngredientSummary {
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
pub struct IngredientSummaryData {
    product: ProductSummary,
}

impl From<&Row> for IngredientSummaryData {
    fn from(row: &Row) -> Self {
        Self {
            product: row.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ProductSummary {
    id: Uuid,
    data: ProductSummaryData,
}

impl From<&Row> for ProductSummary {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("product_id"),
            data: row.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ProductSummaryData {
    name: String,
}

impl From<&Row> for ProductSummaryData {
    fn from(row: &Row) -> Self {
        Self {
            name: row.get("product_name"),
        }
    }
}

#[derive(Serialize)]
pub struct Ingredient {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: IngredientData,
}

impl From<&Row> for Ingredient {
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

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct IngredientMinimal {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: IngredientMinimalData,
}

impl From<&Row> for IngredientMinimal {
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
pub struct IngredientMinimalData {
    product: ProductMinimal,
}

impl From<&Row> for IngredientMinimalData {
    fn from(row: &Row) -> Self {
        Self {
            product: row.into(),
        }
    }
}

#[derive(Serialize)]
pub struct ProductMinimal {
    id: Uuid,
}

impl From<&Row> for ProductMinimal {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("product_id"),
        }
    }
}

#[derive(Deserialize)]
pub struct IngredientNew {
    product: ProductNew,
}

#[derive(Deserialize)]
pub struct ProductNew {
    id: Uuid,
}

#[derive(Deserialize)]
pub struct IngredientUpdate {
    product: Option<ProductUpdate>,
}

#[derive(Deserialize)]
pub struct ProductUpdate {
    id: Option<Uuid>,
}

pub struct DbIngredientsPostgres {
    connection: Object<Manager>,
}

impl DbIngredientsPostgres {
    pub fn new(connection: Object<Manager>) -> Self {
        Self { connection }
    }
}

impl DbIngredients for DbIngredientsPostgres {
    async fn get(&mut self, collection_id: &Uuid) -> Result<Vec<IngredientSummary>> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT
                    ingredients.id,
                    ingredients.ingredient_collection_id,
                    ingredients.product_id,
                    ingredients.ts_created,
                    ingredients.ts_updated,
                    products.name AS product_name

                FROM public.ingredients
                    LEFT JOIN public.products
                        ON ingredients.product_id = products.id

                WHERE ingredients.ingredient_collection_id = $1
                ORDER BY products.name, ingredients.id
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let ingredients = self
            .connection
            .query(&stmt, &[collection_id])
            .await?
            .iter()
            .map(|row| row.into())
            .collect();

        Ok(ingredients)
    }

    async fn get_by_id(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<Ingredient> {
        tracing::debug!("preparing cached statement");
        let stmt = self
            .connection
            .prepare_cached(
                "
                SELECT
                    ingredients.id,
                    ingredients.ingredient_collection_id,
                    ingredients.product_id,
                    ingredients.ts_created,
                    ingredients.ts_updated,
                    products.name AS product_name

                FROM public.ingredients
                    LEFT JOIN public.products
                        ON ingredients.product_id = products.id

                WHERE ingredients.ingredient_collection_id = $1 AND ingredients.id = $2
                ORDER BY products.name
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let ingredient = match self
            .connection
            .query(&stmt, &[collection_id, id])
            .await?
            .iter()
            .map(|row| Ingredient::from(row))
            .collect::<Vec<_>>()
        {
            ingredients if ingredients.len() == 0 => return Err(DbError::NotFound.into()),
            ingredients if ingredients.len() >= 2 => return Err(DbError::TooMany.into()),
            mut ingredients => ingredients.pop().unwrap(),
        };

        Ok(ingredient)
    }

    async fn create(
        &mut self,
        collection_id: &Uuid,
        ingredients: &Vec<IngredientNew>,
    ) -> Result<Vec<IngredientMinimal>> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        let mut inserted = Vec::new();
        for ingredient in ingredients {
            let ingredient_id = Uuid::new_v4();

            tracing::debug!("preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    INSERT INTO public.ingredients (
                        id,
                        ingredient_collection_id,
                        product_id
                    )
                    VALUES (
                        $1, $2, $3
                    )
                    RETURNING ts_created
                    ",
                )
                .await?;

            tracing::debug!("executing query");
            let row = transaction
                .query_one(
                    &stmt,
                    &[&ingredient_id, collection_id, &ingredient.product.id],
                )
                .await?;

            inserted.push(IngredientMinimal {
                id: ingredient_id,
                ts_created: row.get("ts_created"),
                ts_updated: None,
                data: IngredientMinimalData {
                    product: ProductMinimal {
                        id: ingredient.product.id,
                    },
                },
            });
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(inserted)
    }

    async fn update(
        &mut self,
        collection_id: &Uuid,
        id: &Uuid,
        ingredient: &IngredientUpdate,
    ) -> Result<IngredientMinimal> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("get current: preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                SELECT
                    id,
                    ts_created,
                    ts_updated,
                    ingredient_collection_id,
                    product_id
                FROM public.ingredients
                WHERE ingredient_collection_id = $1 AND id = $2
                ",
            )
            .await?;

        tracing::debug!("get current: executing query");
        let current = match transaction.query(&stmt, &[collection_id, id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        tracing::debug!("update: executing query");
        let stmt = transaction
            .prepare_cached(
                "
                UPDATE public.ingredients
                SET product_id = $3,
                    ts_updated = CURRENT_TIMESTAMP
                WHERE ingredient_collection_id = $1 AND id = $2
                RETURNING ts_updated
                ",
            )
            .await?;

        let product_id = ingredient
            .product
            .as_ref()
            .and_then(|product| product.id)
            .unwrap_or(current.get("product_id"));

        tracing::debug!("update: executing query");
        let updated = match transaction
            .query(&stmt, &[collection_id, id, &product_id])
            .await?
        {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(IngredientMinimal {
            id: *id,
            ts_created: current.get("ts_created"),
            ts_updated: updated.get("ts_updated"),
            data: IngredientMinimalData {
                product: ProductMinimal { id: product_id },
            },
        })
    }

    async fn delete(&mut self, collection_id: &Uuid, id: &Uuid) -> Result<()> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                DELETE FROM public.ingredients
                WHERE ingredient_collection_id = $1 AND id = $2
                ",
            )
            .await?;

        tracing::debug!("executing query");
        match transaction.execute(&stmt, &[collection_id, id]).await? {
            rows if rows == 0 => return Err(DbError::NotFound.into()),
            rows if rows >= 2 => return Err(DbError::TooMany.into()),
            _ => (),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(())
    }
}
