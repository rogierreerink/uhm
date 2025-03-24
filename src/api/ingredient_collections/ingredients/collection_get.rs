use crate::db::ingredients::query;
use crate::db::{ingredient_collections, DbError};
use crate::global::AppState;
use crate::types::payloads::{collection, resource};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    product_link: ProductLink,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLink {
    id: Uuid,
    data: ProductData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductData {
    name: String,
}

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<Uuid>,
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

    tracing::debug!("querying parent collection");
    match ingredient_collections::query::query_one(&transaction, &collection_id).await {
        Ok(_) => {}
        Err(err) if err == DbError::NotFound => {
            tracing::error!("parent collection could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            tracing::error!("failed to query parent collection: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("querying collection");
    let data = match query::query(&transaction, &collection_id).await {
        Ok(items) => items
            .iter()
            .map(|item| resource::GetResponse {
                id: item.id,
                created: item.ts_created,
                updated: item.ts_updated,
                data: Resource {
                    product_link: ProductLink {
                        id: item.product.id,
                        data: ProductData {
                            name: item.product.name.clone(),
                        },
                    },
                },
            })
            .collect(),
        Err(err) => {
            tracing::error!("failed to query collection: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(collection::GetResponse {
        pagination: None,
        data,
    }))
}
