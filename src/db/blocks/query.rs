use crate::db::DbError;
use crate::types::error::Error;
use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub kind: Kind,
}

impl From<&Row> for Block {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            kind: row.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    IngredientCollection {
        id: Uuid,
        ingredient_collection: IngredientCollection,
    },
    Paragraph {
        id: Uuid,
        text: String,
    },
}

impl From<&Row> for Kind {
    fn from(row: &Row) -> Self {
        if let Some(id) = row.get("ingredient_collection_block_id") {
            Self::IngredientCollection {
                id,
                ingredient_collection: row.into(),
            }
        } else if let Some(id) = row.get("paragraph_block_id") {
            Self::Paragraph {
                id,
                text: row.get("paragraph_block_text"),
            }
        } else {
            panic!()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct IngredientCollection {
    pub id: Uuid,
}

impl From<&Row> for IngredientCollection {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("ingredient_collection_id"),
        }
    }
}

pub async fn query_blocks<'a>(
    transaction: &'a Transaction<'a>,
) -> Result<Vec<Block>, Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(include_str!("sql/query_blocks.sql"))
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("querying database");
    match transaction.query(&stmt, &[]).await {
        Ok(rows) => Ok(rows.into_iter().map(|row| (&row).into()).collect()),
        Err(err) => return Err(err.into()),
    }
}

pub async fn query_block<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<Block, Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(include_str!("sql/query_block.sql"))
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("querying database");
    match transaction.query(&stmt, &[id]).await {
        Ok(rows) if rows.len() == 0 => Err(DbError::NotFound.into()),
        Ok(rows) if rows.len() >= 2 => Err(DbError::TooMany.into()),
        Ok(rows) => Ok((&rows[0]).into()),
        Err(err) => return Err(err.into()),
    }
}
