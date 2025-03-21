use std::sync::Arc;

use axum::{
    extract::{Path, State},
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
struct Unit {
    id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Quantity {
    size: f64,
    unit: Option<Unit>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    id: Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientRequest {
    product: Product,
    quantities: Option<Vec<Quantity>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IngredientResponse {
    id: Uuid,
    product: Product,
    quantities: Vec<Quantity>,
}

#[axum::debug_handler]
#[instrument(skip(state, ingredients))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(recipe_slug): Path<String>,
    Json(ingredients): Json<Vec<IngredientRequest>>,
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

    let get_recipe_stmt = match transaction
        .prepare_cached(
            "
            SELECT recipe_id AS id
            FROM public.recipes_intl_attrs
            WHERE slug = $1
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

    let recipe_id = match transaction
        .query_one(&get_recipe_stmt, &[&recipe_slug])
        .await
    {
        Ok(row) => row.get::<_, Uuid>("id"),
        Err(err) => {
            tracing::warn!("Recipe could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let mut inserted_ingredients = Vec::new();
    for ingredient in ingredients {
        let insert_ingredient_stmt = match transaction
            .prepare_cached(
                "
                INSERT INTO public.ingredients (
                    id, recipe_id, product_id
                ) VALUES (
                    $1, $2, $3
                )
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

        let ingredient_id = Uuid::new_v4();
        if let Err(err) = transaction
            .execute(
                &insert_ingredient_stmt,
                &[&ingredient_id, &recipe_id, &ingredient.product.id],
            )
            .await
        {
            tracing::error!("Failed to insert ingredient: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let quantities = ingredient.quantities.unwrap_or(vec![]);
        for (order, quantity) in quantities.iter().enumerate().collect::<Vec<_>>() {
            let insert_quantity_stmt = match transaction
                .prepare_cached(
                    "
                    INSERT INTO public.ingredient_quantities (
                        id, ingredient_id, quantity, unit_id, sort_order
                    ) VALUES (
                        $1, $2, $3, $4, $5
                    )
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

            let quantity_id = Uuid::new_v4();
            if let Err(err) = transaction
                .execute(
                    &insert_quantity_stmt,
                    &[
                        &quantity_id,
                        &ingredient_id,
                        &quantity.size,
                        &quantity.unit.as_ref().map(|unit| unit.id),
                        &(order as i32),
                    ],
                )
                .await
            {
                tracing::error!("Failed to insert quantity: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }

        inserted_ingredients.push(IngredientResponse {
            id: ingredient_id,
            product: ingredient.product,
            quantities,
        })
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(inserted_ingredients))
}
