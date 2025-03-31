use crate::db::DbError;
use crate::types::error::Error;
use chrono::Local;
use deadpool_postgres::Transaction;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Resource {
    pub id: Uuid,
    pub ingredient_collection_id: Uuid,
    pub product_id: Uuid,
}

impl From<&Row> for Resource {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ingredient_collection_id: row.get("ingredient_collection_id"),
            product_id: row.get("product_id"),
        }
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
            INSERT INTO public.ingredients (
                id,
                ingredient_collection_id,
                product_id,
                ts_updated
            )
            VALUES ($1, $2, $3, NULL)
            ON CONFLICT (id) DO UPDATE SET
                ingredient_collection_id = EXCLUDED.ingredient_collection_id,
                product_id = EXCLUDED.product_id,
                ts_updated = $4
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction
        .execute(
            &stmt,
            &[
                &item.id,
                &item.ingredient_collection_id,
                &item.product_id,
                &Local::now(),
            ],
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}
