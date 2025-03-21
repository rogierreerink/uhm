use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::instrument;
use uuid::Uuid;

use crate::global::AppState;

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    OriginalUri(uri): OriginalUri,
) -> impl IntoResponse {
    tracing::info!("New request");

    let mut db_conn = match state.db_pool.get().await {
        Ok(db_conn) => db_conn,
        Err(err) => {
            tracing::error!("Failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let get_stmt = match db_conn
        .prepare_cached(
            "
            SELECT id, shopping_list_for_ingredients_id, shopping_list_for_products_id
            FROM public.shopping_list
            WHERE shopping_list.id = $1
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

    let (shopping_list_for_ingredients_id, shopping_list_for_products_id) =
        match db_conn.query_one(&get_stmt, &[&id]).await {
            Ok(row) => (
                row.get::<_, Option<Uuid>>("shopping_list_for_ingredients_id"),
                row.get::<_, Option<Uuid>>("shopping_list_for_products_id"),
            ),
            Err(err) => {
                tracing::warn!("Shopping list item could not be found: {}", err);
                return Ok(());
            }
        };

    let transaction = match db_conn.transaction().await {
        Ok(transaction) => transaction,
        Err(err) => {
            tracing::error!("Failed to prepare transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.shopping_list
                WHERE id = $1;
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => {
            tracing::error!("Failed to prepare transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = transaction.execute(&stmt, &[&id]).await {
        tracing::error!("Failed to delete grocery item: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Some(ref_id) = shopping_list_for_ingredients_id {
        let stmt = match transaction
            .prepare_cached(
                "
                DELETE FROM public.shopping_list_for_ingredients
                    WHERE id = $1;
                ",
            )
            .await
        {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("Failed to prepare transaction: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        if let Err(err) = transaction.execute(&stmt, &[&ref_id]).await {
            tracing::error!("Failed to delete grocery item for ingredient: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Some(ref_id) = shopping_list_for_products_id {
        let stmt = match transaction
            .prepare_cached(
                "
                DELETE FROM public.shopping_list_for_products
                    WHERE id = $1;
                ",
            )
            .await
        {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("Failed to prepare transaction: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        if let Err(err) = transaction.execute(&stmt, &[&ref_id]).await {
            tracing::error!("Failed to delete grocery item for product: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Shopping list item could not be deleted: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let _ = state.change_notifier.send(uri.to_string());

    Ok(())
}
