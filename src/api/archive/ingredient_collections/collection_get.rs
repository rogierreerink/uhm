use crate::db::ingredient_collections::query;
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
    ingredients: Vec<Ingredient>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ingredient {
    id: Uuid,
    data: IngredientData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientData {
    product: Product,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    id: Uuid,
    data: ProductData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductData {
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
                    ingredients: item
                        .ingredients
                        .iter()
                        .map(|ingredient| Ingredient {
                            id: ingredient.id,
                            data: IngredientData {
                                product: Product {
                                    id: ingredient.product.id,
                                    data: ProductData {
                                        name: ingredient.product.name.clone(),
                                    },
                                },
                            },
                        })
                        .collect(),
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
