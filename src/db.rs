use std::error::Error;
use std::fmt::{Debug, Display};

use blocks::{BlockDb, BlockDbPostgres};
use ingredient_collections::{IngredientCollectionDb, IngredientCollectionDbPostgres};
use ingredients::{IngredientDb, IngredientDbPostgres};
use list_items::{ListItemDb, ListItemDbPostgres};
use lists::{ListDb, ListDbPostgres};
use markdown::{MarkdownDb, MarkdownDbPostgres};
use pages::{PageDb, PageDbPostgres};
use products::{ProductDb, ProductDbPostgres};
use sqlx::PgPool;

pub mod blocks;
pub mod ingredient_collections;
pub mod ingredients;
pub mod list_items;
pub mod lists;
pub mod markdown;
pub mod pages;
pub mod products;

pub trait Db {
    fn blocks(&self) -> impl BlockDb;
    fn ingredient_collections(&self) -> impl IngredientCollectionDb;
    fn ingredients(&self) -> impl IngredientDb;
    fn list_items(&self) -> impl ListItemDb;
    fn lists(&self) -> impl ListDb;
    fn markdown(&self) -> impl MarkdownDb;
    fn pages(&self) -> impl PageDb;
    fn products(&self) -> impl ProductDb;
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
    fn blocks(&self) -> impl BlockDb {
        BlockDbPostgres::new(&self.sqlx)
    }

    fn ingredient_collections(&self) -> impl IngredientCollectionDb {
        IngredientCollectionDbPostgres::new(&self.sqlx)
    }

    fn ingredients(&self) -> impl IngredientDb {
        IngredientDbPostgres::new(&self.sqlx)
    }

    fn list_items(&self) -> impl ListItemDb {
        ListItemDbPostgres::new(&self.sqlx)
    }

    fn lists(&self) -> impl ListDb {
        ListDbPostgres::new(&self.sqlx)
    }

    fn markdown(&self) -> impl MarkdownDb {
        MarkdownDbPostgres::new(&self.sqlx)
    }

    fn pages(&self) -> impl PageDb {
        PageDbPostgres::new(&self.sqlx)
    }

    fn products(&self) -> impl ProductDb {
        ProductDbPostgres::new(&self.sqlx)
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
