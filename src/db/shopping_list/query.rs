use crate::db::DbError;
use crate::types::error::Error;
use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Resource {
    // Metadata
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,

    // Data
    pub in_cart: bool,
    pub source: Source,
}

impl From<&Row> for Resource {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            in_cart: row.get("in_cart"),
            source: row.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Source {
    ProductLink { id: Uuid, product: Product },
    Temporary { id: Uuid, name: String },
}

impl From<&Row> for Source {
    fn from(row: &Row) -> Self {
        if let Some(link_id) = row.get("product_link_id") {
            Self::ProductLink {
                id: link_id,
                product: row.into(),
            }
        } else {
            Self::Temporary {
                id: row.get("temp_item_id"),
                name: row.get("temp_item_name"),
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
}

impl From<&Row> for Product {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("product_id"),
            name: row.get("product_name"),
        }
    }
}

pub async fn query<'a>(
    transaction: &'a Transaction<'a>,
) -> Result<Vec<Resource>, Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(include_str!("sql/query.sql"))
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

pub async fn query_one<'a>(
    transaction: &'a Transaction<'a>,
    id: Uuid,
) -> Result<Resource, Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(include_str!("sql/query_one.sql"))
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("querying database");
    match transaction.query(&stmt, &[&id]).await {
        Ok(rows) if rows.len() == 0 => Err(DbError::NotFound.into()),
        Ok(rows) if rows.len() >= 2 => Err(DbError::TooMany.into()),
        Ok(rows) => Ok((&rows[0]).into()),
        Err(err) => return Err(err.into()),
    }
}
