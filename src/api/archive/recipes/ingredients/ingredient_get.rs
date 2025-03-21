use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use tracing::instrument;
use uuid::Uuid;

use crate::{api::plurals::Plural, global::AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Unit {
    id: Uuid,
    name: Plural,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Quantity {
    size: f64,
    unit: Option<Unit>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    id: Uuid,
    name: Plural,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Ingredient {
    id: Uuid,
    product: Product,
    quantities: Vec<Quantity>,
}

#[instrument(skip(state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path((recipe_slug, ingredient_id)): Path<(String, Uuid)>,
) -> impl IntoResponse {
    tracing::info!("New request");

    let db_conn = match state.db_pool.get().await {
        Ok(db_conn) => db_conn,
        Err(err) => {
            tracing::error!("Failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let stmt = match db_conn
        .prepare_cached(
            "
            SELECT
                products_intl_attrs.id AS product_id,
                product_names.zero AS product_name_zero,
                product_names.one AS product_name_one,
                product_names.two AS product_name_two,
                product_names.few AS product_name_few,
                product_names.many AS product_name_many,
                product_names.other AS product_name_other,
                quantities.id AS quantity_id,
                quantities.quantity AS quantity,
                quantity_units.id AS unit_id,
                quantity_unit_names.zero AS quantity_unit_name_zero,
                quantity_unit_names.one AS quantity_unit_name_one,
                quantity_unit_names.two AS quantity_unit_name_two,
                quantity_unit_names.few AS quantity_unit_name_few,
                quantity_unit_names.many AS quantity_unit_name_many,
                quantity_unit_names.other AS quantity_unit_name_other

            FROM public.recipes_intl_attrs AS recipes_intl_attrs
                JOIN public.ingredients AS ingredients
                    ON recipes_intl_attrs.recipe_id = ingredients.recipe_id

                JOIN public.products_intl_attrs AS products_intl_attrs
                    ON ingredients.product_id = products_intl_attrs.product_id
                JOIN public.plurals_intl AS product_names
                    ON products_intl_attrs.name_plural_id = product_names.id

                LEFT JOIN public.ingredient_quantities AS quantities
                    ON ingredients.id = quantities.ingredient_id
                LEFT JOIN public.units AS quantity_units
                    ON quantities.unit_id = quantity_units.id
                LEFT JOIN public.units_intl_attrs AS quantity_unit_intl_attrs
                    ON quantity_units.id = quantity_unit_intl_attrs.unit_id
                LEFT JOIN public.plurals_intl AS quantity_unit_names
                    ON quantity_unit_intl_attrs.name_plural_id = quantity_unit_names.id

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

    let ingredient = match db_conn.query(&stmt, &[&recipe_slug, &ingredient_id]).await {
        Ok(rows) if rows.len() == 0 => {
            tracing::warn!("Ingredient could not be found.");
            return Err(StatusCode::NOT_FOUND);
        }
        Ok(rows) => {
            let mut ingredient = Ingredient {
                id: ingredient_id,
                product: Product {
                    id: rows[0].get("product_id"),
                    name: Plural {
                        zero: rows[0].get("product_name_zero"),
                        one: rows[0].get("product_name_one"),
                        two: rows[0].get("product_name_two"),
                        few: rows[0].get("product_name_few"),
                        many: rows[0].get("product_name_many"),
                        other: rows[0].get("product_name_other"),
                    },
                },
                quantities: Vec::new(),
            };

            for row in rows {
                if let Some(quantity) = row.get("quantity") {
                    ingredient.quantities.push(Quantity {
                        size: quantity,
                        unit: match row.get("unit_id") {
                            Some(unit_id) => Some(Unit {
                                id: unit_id,
                                name: Plural {
                                    zero: row.get("quantity_unit_name_zero"),
                                    one: row.get("quantity_unit_name_one"),
                                    two: row.get("quantity_unit_name_two"),
                                    few: row.get("quantity_unit_name_few"),
                                    many: row.get("quantity_unit_name_many"),
                                    other: row.get("quantity_unit_name_other"),
                                },
                            }),
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

    Ok(Json(ingredient))
}
