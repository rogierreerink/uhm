use std::sync::Arc;

use axum::{
    extract::{OriginalUri, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::global::AppState;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShoppingListItem {
    id: Option<Uuid>,
    in_cart: Option<bool>,
}

#[axum::debug_handler]
#[instrument(skip(body, state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    OriginalUri(uri): OriginalUri,
    Json(body): Json<ShoppingListItem>,
) -> impl IntoResponse {
    tracing::info!("New request");

    let db_conn = match state.db_pool.get().await {
        Ok(db_conn) => db_conn,
        Err(err) => {
            tracing::error!("Failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let get_stmt = match db_conn
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

    if let Err(err) = db_conn.query_one(&get_stmt, &[&id]).await {
        tracing::warn!("Shopping list item could not be found: {}", err);
        return Err(StatusCode::NOT_FOUND);
    }

    let patch_stmt = match db_conn
        .prepare_cached(
            "
            UPDATE public.shopping_list
            SET in_cart = COALESCE($2, in_cart)
            WHERE id = $1
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

    if let Err(err) = db_conn.query(&patch_stmt, &[&id, &body.in_cart]).await {
        tracing::warn!("Shopping list item could not be patched: {}", err);
        return Err(StatusCode::NOT_FOUND);
    }

    let shopping_list_item = match db_conn.query_one(&get_stmt, &[&id]).await {
        Ok(row) => ShoppingListItem {
            id: row.get("id"),
            in_cart: row.get("in_cart"),
        },
        Err(err) => {
            tracing::warn!("Shopping list item could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let _ = state.change_notifier.send(uri.to_string());

    Ok(Json(shopping_list_item))
}
