use std::error::Error;
use std::fmt::{Debug, Display};

use anyhow::Result;
use deadpool::managed::{Object, Pool};
use deadpool_postgres::Manager;

pub mod blocks;
// pub mod ingredient_collections;
// pub mod ingredients;
// pub mod products;
// pub mod shopping_list;

#[trait_variant::make(Send)]
pub trait Db {
    async fn blocks(&self) -> Result<impl blocks::DbBlocks>;
}

pub struct DbPostgres {
    pool: Pool<Manager>,
}

impl DbPostgres {
    pub fn new(pool: Pool<Manager>) -> Self {
        Self { pool }
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbError {
    NotFound,
    TooMany,
    InvalidOperation,
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::NotFound => write!(f, "resource could not be found"),
            DbError::TooMany => write!(f, "query returned too many results"),
            DbError::InvalidOperation => write!(f, "operation may not be performed"),
        }
    }
}

impl Error for DbError {}
