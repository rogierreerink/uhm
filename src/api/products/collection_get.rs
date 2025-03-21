use crate::db::products::query;
use crate::global::AppState;
use crate::types::payloads::{CollectionResponse, ResourceResponse};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Resource {
    name: String,
    shopping_list_item_links: Vec<ShoppingListItemLink>,
}

#[derive(Serialize)]
pub struct ShoppingListItemLink {
    id: Uuid,
}

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Query(search_query): Query<query::SearchQuery>,
) -> impl IntoResponse {
    tracing::debug!("setting up database connection");
    let mut connection = match state.db_pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("starting database transaction");
    let transaction = match connection.transaction().await {
        Ok(transaction) => transaction,
        Err(err) => {
            tracing::error!("failed to start a database transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("querying collection");
    let data = match query::query(&transaction, &search_query).await {
        Ok(items) => items
            .iter()
            .map(|item| ResourceResponse {
                id: item.id,
                created: item.ts_created,
                updated: item.ts_updated,
                data: Resource {
                    name: item.name.clone(),
                    shopping_list_item_links: item
                        .shopping_list_item_links
                        .iter()
                        .map(|link| ShoppingListItemLink { id: link.id })
                        .collect(),
                },
            })
            .collect(),
        Err(err) => {
            tracing::error!("failed to query collection: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(CollectionResponse {
        pagination: None,
        data,
    }))
}
