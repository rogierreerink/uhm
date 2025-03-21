use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use json_patch::Patch;
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
    id: Option<Uuid>, // Should not be set by the client
    size: f64,
    unit: Option<Unit>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Ingredient {
    id: Uuid,
    product: Product,
    quantities: Vec<Quantity>,
}

#[axum::debug_handler]
#[instrument(skip(state, payload))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path((recipe_slug, ingredient_id)): Path<(String, Uuid)>,
    Json(payload): Json<Patch>,
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
            tracing::error!("Failed to start transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let get_stmt = match transaction
        .prepare_cached(
            "
            SELECT
                ingredients.product_id AS product_id,
                quantities.id AS quantity_id,
                quantities.quantity AS quantity,
                quantities.unit_id AS unit_id

            FROM public.recipes_intl_attrs AS recipes_intl_attrs
                JOIN public.ingredients AS ingredients
                    ON recipes_intl_attrs.recipe_id = ingredients.recipe_id
                LEFT JOIN public.ingredient_quantities AS quantities
                    ON ingredients.id = quantities.ingredient_id

            WHERE
                recipes_intl_attrs.slug = $1 AND
                ingredients.id = $2

            -- Must be equal for each GET and PATCH endpoint
            ORDER BY quantities.sort_order, quantities.id
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

    let current_state = match transaction
        .query(&get_stmt, &[&recipe_slug, &ingredient_id])
        .await
    {
        Ok(rows) if rows.len() == 0 => {
            tracing::warn!("Ingredient could not be found.");
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(rows) => {
            let mut ingredient = Ingredient {
                id: ingredient_id,
                product: Product {
                    id: rows[0].get("product_id"),
                },
                quantities: Vec::new(),
            };

            for row in rows {
                if let Some(quantity) = row.get("quantity") {
                    ingredient.quantities.push(Quantity {
                        id: row.get("quantity_id"),
                        size: quantity,
                        unit: match row.get("unit_id") {
                            Some(unit_id) => Some(Unit { id: unit_id }),
                            None => None,
                        },
                    })
                }
            }

            ingredient
        }
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut state_json = match serde_json::to_value(&current_state) {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("Failed to serialize ingredient: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = json_patch::patch(&mut state_json, &payload) {
        tracing::error!("Failed to patch ingredient: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }
    println!("{:#?}", state_json);

    let new_state = match serde_json::from_value::<Ingredient>(state_json) {
        Ok(state) if state.id != current_state.id => {
            tracing::warn!("Client tried to alter the ingredient ID");
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(state) => state,
        Err(err) => {
            tracing::warn!("Failed to deserialize patch result: {}", err);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if new_state.product.id != current_state.product.id {
        tracing::info!("Patching product reference");

        let update_product_stmt = match transaction
            .prepare_cached(
                "
                UPDATE public.ingredients
                SET product_id = $2
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

        match transaction
            .execute(
                &update_product_stmt,
                &[&ingredient_id, &new_state.product.id],
            )
            .await
        {
            Ok(update_count) if update_count == 0 => {
                tracing::error!("Product reference was not updated");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(_) => (),
            Err(err) => {
                tracing::error!("Product reference could not be updated: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
    }

    let deleted_quantities: Vec<_> = current_state
        .quantities
        .iter()
        .filter(|q| {
            q.id.is_some_and(|q_id| {
                new_state
                    .quantities
                    .iter()
                    .all(|new_q| new_q.id.is_some_and(|new_q_id| q_id != new_q_id))
            })
        })
        .collect();
    for quantity in deleted_quantities {
        let delete_quantity_stmt = match transaction
            .prepare_cached(
                "
                DELETE FROM public.ingredient_quantities
                WHERE id = $1
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

        if let Err(err) = transaction
            .execute(&delete_quantity_stmt, &[&quantity.id.unwrap()])
            .await
        {
            tracing::error!("Failed to delete quantity: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    for (order, quantity) in new_state.quantities.iter().enumerate().collect::<Vec<_>>() {
        let insert_quantity_stmt = match transaction
            .prepare_cached(
                "
                INSERT INTO public.ingredient_quantities (
                    id, ingredient_id, quantity, unit_id, sort_order
                ) VALUES (
                    $1, $2, $3, $4, $5
                )
                ON CONFLICT (id) DO UPDATE SET
                    quantity = EXCLUDED.quantity,
                    unit_id = EXCLUDED.unit_id,
                    sort_order = EXCLUDED.sort_order
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

        if let Err(err) = transaction
            .execute(
                &insert_quantity_stmt,
                &[
                    &quantity.id.unwrap_or(Uuid::new_v4()),
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

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
}
