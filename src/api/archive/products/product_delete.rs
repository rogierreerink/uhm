use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::instrument;
use uuid::Uuid;

use crate::global::AppState;

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn handle(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    tracing::info!("New request");

    let mut db_conn = match state.db_pool.get().await {
        Ok(db_conn) => db_conn,
        Err(err) => {
            tracing::error!("Failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let transaction = match db_conn.transaction().await {
        Ok(transaction) => transaction,
        Err(err) => {
            tracing::error!("Failed to start transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let get_stmt = match transaction
        .prepare_cached(
            "
            SELECT product_id, id AS attrs_id, name_plural_id
            FROM public.products_intl_attrs
            WHERE product_id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => {
            tracing::error!("Failed to prepare cached statement: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let (product_id, attrs_id, name_plural_id): (Uuid, Uuid, Uuid) =
        match transaction.query_one(&get_stmt, &[&id]).await {
            Ok(row) => (
                row.get("product_id"),
                row.get("attrs_id"),
                row.get("name_plural_id"),
            ),
            Err(err) => {
                tracing::warn!("Product could not be found: {}", err);
                return Err(StatusCode::NOT_FOUND);
            }
        };

    let delete_intl_attrs_stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.products_intl_attrs
            WHERE id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => {
            tracing::error!("Failed to prepare statement: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = transaction
        .execute(&delete_intl_attrs_stmt, &[&attrs_id])
        .await
    {
        tracing::error!("Failed to delete intl attrs: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let delete_product_stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.products
            WHERE id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => {
            tracing::error!("Failed to prepare statement: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = transaction
        .execute(&delete_product_stmt, &[&product_id])
        .await
    {
        tracing::error!("Failed to delete product: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let delete_name_plural_stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.plurals_intl
            WHERE id = $1
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => {
            tracing::error!("Failed to prepare statement: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = transaction
        .execute(&delete_name_plural_stmt, &[&name_plural_id])
        .await
    {
        tracing::error!("Failed to delete plural name: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
}
