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

pub trait Db {
    fn blocks(&self) -> Result<impl BlockDb>;
    fn ingredient_collections(&self) -> Result<impl IngredientCollectionDb>;
    fn ingredients(&self) -> Result<impl IngredientDb>;
    fn list_items(&self) -> Result<impl ListItemDb>;
    fn markdown(&self) -> Result<impl MarkdownDb>;
    fn products(&self) -> Result<impl ProductDb>;
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
    fn blocks(&self) -> Result<impl BlockDb> {
        Ok(blocks::BlockDbPostgres::new(&self.sqlx))
    }

    fn ingredient_collections(&self) -> Result<impl IngredientCollectionDb> {
        Ok(ingredient_collections::IngredientCollectionDbPostgres::new(
            &self.sqlx,
        ))
    }

    fn ingredients(&self) -> Result<impl IngredientDb> {
        Ok(ingredients::IngredientDbPostgres::new(&self.sqlx))
    }

    fn list_items(&self) -> Result<impl ListItemDb> {
        Ok(list_items::ListItemDbPostgres::new(&self.sqlx))
    }

    fn markdown(&self) -> Result<impl MarkdownDb> {
        Ok(markdown::MarkdownDbPostgres::new(&self.sqlx))
    }

    fn products(&self) -> Result<impl ProductDb> {
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
