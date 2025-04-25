use std::error::Error;
use std::fmt::{Debug, Display};

use anyhow::Result;
use blocks::BlockDb;
use ingredient_collections::IngredientCollectionDb;
use ingredients::IngredientDb;
use list_items::ListItemDb;
use markdown::MarkdownDb;
use products::ProductDb;
use sqlx::PgPool;

pub mod blocks;
pub mod ingredient_collections;
pub mod ingredients;
pub mod list_items;
pub mod markdown;
pub mod products;

#[trait_variant::make(Send)]
pub trait Db {
    async fn blocks(&self) -> Result<impl BlockDb>;
    async fn ingredient_collections(&self) -> Result<impl IngredientCollectionDb>;
    async fn ingredients(&self) -> Result<impl IngredientDb>;
    async fn list_items(&self) -> Result<impl ListItemDb>;
    async fn markdown(&self) -> Result<impl MarkdownDb>;
    async fn products(&self) -> Result<impl ProductDb>;
}

pub struct DbPostgres {
    sqlx: sqlx::PgPool,
}

impl DbPostgres {
    pub fn new(sqlx: PgPool) -> Self {
        Self { sqlx }
    }
}

impl Db for DbPostgres {
    async fn blocks(&self) -> Result<impl BlockDb> {
        Ok(blocks::BlockDbPostgres::new(&self.sqlx))
    }

    async fn ingredient_collections(&self) -> Result<impl IngredientCollectionDb> {
        Ok(ingredient_collections::IngredientCollectionDbPostgres::new(
            &self.sqlx,
        ))
    }

    async fn ingredients(&self) -> Result<impl IngredientDb> {
        Ok(ingredients::IngredientDbPostgres::new(&self.sqlx))
    }

    async fn list_items(&self) -> Result<impl ListItemDb> {
        Ok(list_items::ListItemDbPostgres::new(&self.sqlx))
    }

    async fn markdown(&self) -> Result<impl MarkdownDb> {
        Ok(markdown::MarkdownDbPostgres::new(&self.sqlx))
    }

    async fn products(&self) -> Result<impl ProductDb> {
        Ok(products::ProductDbPostgres::new(&self.sqlx))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbError {
    NotFound,
    InvalidOperation,
    InvalidContent,
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "resource could not be found"),
            DbError::InvalidOperation => write!(f, "operation may not be performed"),
            DbError::InvalidContent => write!(f, "invalid database contents"),
        }
    }
}

impl Error for DbError {}
