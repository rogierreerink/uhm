use std::sync::Arc;

use axum::{
    extract::{OriginalUri, State},
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
pub struct Ingredient {
    id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ShoppingListItem {
    IngredientLink { ingredient: Ingredient },
    ProductLink { product: Product },
}

#[axum::debug_handler]
#[instrument(skip(body, state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    OriginalUri(uri): OriginalUri,
    Json(body): Json<Vec<ShoppingListItem>>,
) -> impl IntoResponse {
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
            tracing::error!("Failed to prepare transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    for shopping_list_item in body {
        match shopping_list_item {
            ShoppingListItem::IngredientLink { ingredient } => {
                let stmt = match transaction
                    .prepare_cached(
                        "
                        INSERT INTO public.shopping_list_for_ingredients (
                            id, ingredient_id
                        ) VALUES ($1, $2);
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

                let ref_id = Uuid::new_v4();
                match transaction.execute(&stmt, &[&ref_id, &ingredient.id]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("Failed to create grocery item for ingredient: {}", err);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }

                let stmt = match transaction
                    .prepare_cached(
                        "
                        INSERT INTO public.shopping_list (
                            id, shopping_list_for_ingredients_id
                        ) VALUES ($1, $2);
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

                let id = Uuid::new_v4();
                match transaction.execute(&stmt, &[&id, &ref_id]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("Failed to create grocery item: {}", err);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
            ShoppingListItem::ProductLink { product } => {
                let stmt = match transaction
                    .prepare_cached(
                        "
                        INSERT INTO public.shopping_list_for_products (
                            id, product_id
                        ) VALUES ($1, $2);
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

                let ref_id = Uuid::new_v4();
                match transaction.execute(&stmt, &[&ref_id, &product.id]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("Failed to create grocery item for product: {}", err);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }

                let stmt = match transaction
                    .prepare_cached(
                        "
                        INSERT INTO public.shopping_list (
                            id, shopping_list_for_products_id
                        ) VALUES ($1, $2);
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

                let id = Uuid::new_v4();
                match transaction.execute(&stmt, &[&id, &ref_id]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("Failed to create grocery item: {}", err);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
            }
        }
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Shopping list items could not be created: {}", err);
        return Err(StatusCode::NOT_FOUND);
    }

    // let shopping_list = match db_conn.query(&stmt, &[]).await {
    //     Ok(_) => Vec::<ShoppingListItem>::new(),
    //     Err(err) => {
    //         tracing::warn!("Shopping list item could not be found: {}", err);
    //         return Err(StatusCode::NOT_FOUND);
    //     }
    // };

    let _ = state.change_notifier.send(uri.to_string());

    Ok(Json(Vec::<ShoppingListItem>::new()))
}
