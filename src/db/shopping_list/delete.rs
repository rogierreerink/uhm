use crate::db::DbError;
use crate::types::error::Error;
use deadpool_postgres::Transaction;
use uuid::Uuid;

pub async fn delete<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.shopping_list
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

pub enum SourceId {
    ProductLink(Uuid),
    Temporary(Uuid),
}

pub async fn delete_source<'a>(
    transaction: &'a Transaction<'a>,
    id: &SourceId,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    match id {
        SourceId::ProductLink(id) => delete_product_link(transaction, id).await,
        SourceId::Temporary(id) => delete_temporary(transaction, id).await,
    }
}

async fn delete_temporary<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.shopping_list_temp_items
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

async fn delete_product_link<'a>(
    transaction: &'a Transaction<'a>,
    id: &Uuid,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.shopping_list_product_links
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
