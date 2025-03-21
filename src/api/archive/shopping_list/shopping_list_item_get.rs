use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use tracing::instrument;
use uuid::Uuid;

use crate::global::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShoppingListItem {
    id: Uuid,
    in_cart: bool,
}

#[instrument(skip(state))]
pub async fn handle(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
    tracing::info!("New request");

    let db_conn = match state.db_pool.get().await {
        Ok(db_conn) => db_conn,
        Err(err) => {
            tracing::error!("Failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let stmt = match db_conn
        .prepare_cached(
            "
            SELECT
                shopping_list.id AS id,
                shopping_list.in_cart AS in_cart
            FROM public.shopping_list AS shopping_list
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

    let shopping_list_item = match db_conn.query_one(&stmt, &[&id]).await {
        Ok(row) => ShoppingListItem {
            id: row.get("id"),
            in_cart: row.get("in_cart"),
        },
        Err(err) => {
            tracing::warn!("Shopping list item could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    Ok(Json(shopping_list_item))
}
