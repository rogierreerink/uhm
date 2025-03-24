use crate::types::error::Error;
use crate::{db::DbError, utilities::group::GroupIterExt};
use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Resource {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub ingredients: Vec<Ingredient>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ingredient {
    pub id: Uuid,
    pub product: Product,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
}

struct QueryResult(Vec<Resource>);

impl QueryResult {
    fn inner(self) -> Vec<Resource> {
        self.0
    }
}

impl From<Vec<Row>> for QueryResult {
    fn from(rows: Vec<Row>) -> Self {
        Self(
            rows.into_iter()
                .group_map(
                    |i| i.get::<_, Uuid>("id"),
                    |g| {
                        g.fold(None, |resource, row| {
                            let mut resource = resource.unwrap_or_else(|| Resource {
                                id: row.get("id"),
                                ts_created: row.get("ts_created"),
                                ts_updated: row.get("ts_updated"),
                                ingredients: vec![],
                            });

                            if let Some(id) = row.get("ingredient_id") {
                                resource.ingredients.push(Ingredient {
                                    id,
                                    product: Product {
                                        id: row.get("product_id"),
                                        name: row.get("product_name"),
                                    },
                                });
                            }

                            Some(resource)
                        })
                        .unwrap()
                    },
                )
                .collect(),
        )
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
        Ok(rows) => Ok(QueryResult::from(rows).inner()),
        Err(err) => return Err(err.into()),
    }
}

pub async fn query_one<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
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
    match transaction.query(&stmt, &[id]).await {
        Ok(rows) if rows.len() == 0 => Err(DbError::NotFound.into()),
        Ok(rows) => Ok(QueryResult::from(rows).inner()[0].clone()),
        Err(err) => return Err(err.into()),
    }
}
