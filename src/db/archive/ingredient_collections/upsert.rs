use crate::db::DbError;
use crate::types::error::Error;
use chrono::Local;
use deadpool_postgres::Transaction;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Resource {
    pub id: Uuid,
}

impl From<&Row> for Resource {
    fn from(row: &Row) -> Self {
        Self { id: row.get("id") }
    }
}

pub async fn upsert<'a>(
    transaction: &'a Transaction<'a>,
    item: &Resource,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.ingredient_collections (id, ts_updated)
            VALUES ($1, NULL)
            ON CONFLICT (id) DO UPDATE SET
                ts_updated = $2
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction.execute(&stmt, &[&item.id, &Local::now()]).await {
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}
