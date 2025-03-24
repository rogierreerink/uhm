use crate::db::ingredient_collections::query;
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
    ingredient_links: Vec<IngredientLink>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientLink {
    id: Uuid,
    data: IngredientData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientData {
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
pub async fn handle(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
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
    match query::query_one(&transaction, &id).await {
        Ok(item) => Ok(Json(GetResponse {
            id: item.id,
            created: item.ts_created,
            updated: item.ts_updated,
            data: Resource {
                ingredient_links: item
                    .ingredient_links
                    .iter()
                    .map(|ingredient| IngredientLink {
                        id: ingredient.id,
                        data: IngredientData {
                            product_link: ProductLink {
                                id: ingredient.data.product_link.id,
                                data: ProductData {
                                    name: ingredient.data.product_link.data.name.clone(),
                                },
                            },
                        },
                    })
                    .collect(),
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
