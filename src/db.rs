use std::error::Error;
use std::fmt::{Debug, Display};

use anyhow::Result;
use deadpool::managed::{Object, Pool};
use deadpool_postgres::Manager;
use sqlx::PgPool;

pub mod blocks;
pub mod ingredient_collections;
pub mod ingredients;
pub mod list_items;
pub mod paragraphs;
pub mod products;

#[trait_variant::make(Send)]
pub trait Db {
    async fn blocks(&self) -> Result<impl blocks::DbBlocks>;
    async fn ingredient_collections(
        &self,
    ) -> Result<impl ingredient_collections::IngredientCollectionDb>;
    async fn ingredients(&self) -> Result<impl ingredients::IngredientDb>;
    async fn list_items(&self) -> Result<impl list_items::ListItemDb>;
    async fn paragraphs(&self) -> Result<impl paragraphs::ParagraphDb>;
    async fn products(&self) -> Result<impl products::ProductDb>;
}

pub struct DbPostgres {
    pool: Pool<Manager>,
    sqlx: sqlx::PgPool,
}

impl DbPostgres {
    pub fn new(pool: Pool<Manager>, sqlx: PgPool) -> Self {
        Self { pool, sqlx }
    }

    async fn get_connection(&self) -> Result<Object<Manager>> {
        tracing::debug!("waiting for database connection");
        Ok(self.pool.get().await?)
    }
}

impl Db for DbPostgres {
    async fn blocks(&self) -> Result<impl blocks::DbBlocks> {
        Ok(blocks::DbBlocksPostgres::new(self.get_connection().await?))
    }

    async fn ingredient_collections(
        &self,
    ) -> Result<impl ingredient_collections::IngredientCollectionDb> {
        Ok(ingredient_collections::IngredientCollectionDbPostgres::new(
            &self.sqlx,
        ))
    }

    async fn ingredients(&self) -> Result<impl ingredients::IngredientDb> {
        Ok(ingredients::IngredientDbPostgres::new(&self.sqlx))
    }

    async fn list_items(&self) -> Result<impl list_items::ListItemDb> {
        Ok(list_items::ListItemDbPostgres::new(&self.sqlx))
    }

    async fn paragraphs(&self) -> Result<impl paragraphs::ParagraphDb> {
        Ok(paragraphs::ParagraphDbPostgres::new(&self.sqlx))
    }

    async fn products(&self) -> Result<impl products::ProductDb> {
        Ok(products::ProductDbPostgres::new(&self.sqlx))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbError {
    NotFound,
    TooMany,
    InvalidOperation,
    InvalidContent,
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "resource could not be found"),
            DbError::TooMany => write!(f, "query returned too many results"),
            DbError::InvalidOperation => write!(f, "operation may not be performed"),
            DbError::InvalidContent => write!(f, "invalid database contents"),
        }
    }
}

impl Error for DbError {}
