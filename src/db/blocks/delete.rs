use crate::db::DbError;
use crate::types::error::Error;
use deadpool_postgres::Transaction;
use uuid::Uuid;

pub async fn delete_block<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.blocks
            WHERE id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction.execute(&stmt, &[id]).await {
        Ok(rm) if rm == 0 => Err(DbError::NotFound.into()),
        Ok(rm) if rm >= 2 => Err(DbError::TooMany.into()),
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}

pub async fn delete_ingredient_collection_block<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.ingredient_collection_blocks
            WHERE id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction.execute(&stmt, &[id]).await {
        Ok(rm) if rm == 0 => Err(DbError::NotFound.into()),
        Ok(rm) if rm >= 2 => Err(DbError::TooMany.into()),
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}

pub async fn delete_paragraph_block<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.paragraph_blocks
            WHERE id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction.execute(&stmt, &[id]).await {
        Ok(rm) if rm == 0 => Err(DbError::NotFound.into()),
        Ok(rm) if rm >= 2 => Err(DbError::TooMany.into()),
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}
