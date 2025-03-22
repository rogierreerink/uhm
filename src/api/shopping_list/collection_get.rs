use crate::db::shopping_list::query;
use crate::global::AppState;
use crate::types::payloads::{collection, resource};
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    in_cart: bool,
    source: Source,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Source {
    Product { id: Uuid, data: Product },
    Temporary { data: Temporary },
}

#[derive(Serialize)]
struct Product {
    name: String,
}

#[derive(Serialize)]
struct Temporary {
    name: String,
}

pub async fn handle(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
    let data = match query::query(&transaction).await {
        Ok(items) => items
            .iter()
            .map(|item| resource::GetResponse {
                id: item.id,
                created: item.ts_created,
                updated: item.ts_updated,
                data: Resource {
                    in_cart: item.in_cart,
                    source: match &item.source {
                        query::Source::ProductLink { product, .. } => Source::Product {
                            id: product.id.clone(),
                            data: Product {
                                name: product.name.clone(),
                            },
                        },
                        query::Source::Temporary { name, .. } => Source::Temporary {
                            data: Temporary { name: name.clone() },
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
