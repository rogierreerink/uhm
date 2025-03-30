use deadpool::managed::{Object, Pool};
use deadpool_postgres::Manager;

use crate::types::error::Error;

pub mod blocks;
pub mod blocks2;
pub mod ingredient_collections;
pub mod ingredients;
pub mod products;
pub mod shopping_list;

pub trait Db {
    async fn blocks(&mut self) -> Result<impl blocks2::DbBlocks, DbError>;
}

pub struct DbPostgres {
    pool: Pool<Manager>,
}

impl DbPostgres {
    pub fn new(pool: Pool<Manager>) -> Self {
        Self { pool }
    }

    async fn get_connection(&mut self) -> Result<Object<Manager>, DbError> {
        tracing::debug!("waiting for database connection");
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(err) => {
                tracing::error!("failed to get a connection from the pool: {}", err);
                return Err(DbError::Connection);
            }
        }
    }
}

impl Db for DbPostgres {
    async fn blocks(&mut self) -> Result<impl blocks2::DbBlocks, DbError> {
        Ok(blocks2::DbBlocksPostgres::new(self.get_connection().await?))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DbError {
    Connection,
    Query,
    NotFound,
    TooMany,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbError::Connection => write!(f, "failed to setup database connection"),
            DbError::Query => write!(f, "failed to query database"),
            DbError::NotFound => write!(f, "resource could not be found"),
            DbError::TooMany => write!(f, "query returned too many results"),
        }
    }
}

impl<T> From<DbError> for Error<DbError, T>
where
    T: std::error::Error,
{
    fn from(err: DbError) -> Self {
        Error::new(err)
    }
}

impl<T> From<T> for Error<DbError, T>
where
    T: std::error::Error,
{
    fn from(err: T) -> Error<DbError, T> {
        Error::from_error(err)
    }
}
