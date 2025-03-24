use crate::db::ingredients::query;
use crate::db::DbError;
use crate::global::AppState;
use crate::types::payloads::resource::GetResponse;
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

#[tracing::instrument(skip(state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path((collection_id, id)): Path<(Uuid, Uuid)>,
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

    tracing::debug!("querying resource");
    match query::query_one(&transaction, &collection_id, &id).await {
        Ok(item) => Ok(Json(GetResponse {
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
        })),
        Err(err) if err == DbError::NotFound => {
            tracing::warn!("resource could not be found");
            Err(StatusCode::NOT_FOUND)
        }
        Err(err) => {
            tracing::error!("failed to query resource: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
