use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{api::plurals::Plural, global::AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ShoppingListItem {
    id: Uuid,
    in_cart: bool,
    quantity: Option<f32>,
    name: String,
}

#[instrument(skip(state))]
pub async fn handle(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
                shopping_list.id AS id,
                shopping_list.in_cart AS in_cart,
                products_intl_attrs.id AS product_id,
                product_name_plurals_intl.zero AS product_name_zero,
                product_name_plurals_intl.one AS product_name_one,
                product_name_plurals_intl.two AS product_name_two,
                product_name_plurals_intl.few AS product_name_few,
                product_name_plurals_intl.many AS product_name_many,
                product_name_plurals_intl.other AS product_name_other,
                NULL AS ingredient_id,
                NULL AS quantity,
                NULL AS unit_id,
                NULL AS quantity_name_zero,
                NULL AS quantity_name_one,
                NULL AS quantity_name_two,
                NULL AS quantity_name_few,
                NULL AS quantity_name_many,
                NULL AS quantity_name_other,
                NULL AS recipe_slug,
                NULL AS recipe_name
                
            FROM public.shopping_list AS shopping_list
                JOIN public.shopping_list_for_products AS shopping_list_for_products
                    ON shopping_list.shopping_list_for_products_id = shopping_list_for_products.id

                JOIN public.products_intl_attrs AS products_intl_attrs
                    ON shopping_list_for_products.product_id = products_intl_attrs.product_id
                JOIN public.plurals_intl AS product_name_plurals_intl
                    ON products_intl_attrs.name_plural_id = product_name_plurals_intl.id
                
            UNION SELECT
                shopping_list.id AS id,
                shopping_list.in_cart AS in_cart,
                products_intl_attrs.id AS product_id,
                product_name_plurals_intl.zero AS product_name_zero,
                product_name_plurals_intl.one AS product_name_one,
                product_name_plurals_intl.two AS product_name_two,
                product_name_plurals_intl.few AS product_name_few,
                product_name_plurals_intl.many AS product_name_many,
                product_name_plurals_intl.other AS product_name_other,
                shopping_list_for_ingredients.ingredient_id AS ingredient_id,
                quantities.quantity AS quantity,
                quantity_units.id AS unit_id,
                quantity_unit_names.zero AS quantity_name_zero,
                quantity_unit_names.one AS quantity_name_one,
                quantity_unit_names.two AS quantity_name_two,
                quantity_unit_names.few AS quantity_name_few,
                quantity_unit_names.many AS quantity_name_many,
                quantity_unit_names.other AS quantity_name_other,
                recipes_intl_attrs.slug AS recipe_slug,
                recipes_intl_attrs.name AS recipe_name
                
            FROM public.shopping_list AS shopping_list
                JOIN public.shopping_list_for_ingredients AS shopping_list_for_ingredients
                    ON shopping_list.shopping_list_for_ingredients_id = shopping_list_for_ingredients.id
                JOIN public.ingredients AS ingredients
                    ON shopping_list_for_ingredients.ingredient_id = ingredients.id
                
                JOIN public.products_intl_attrs AS products_intl_attrs
                    ON ingredients.product_id = products_intl_attrs.product_id
                JOIN public.plurals_intl AS product_name_plurals_intl
                    ON products_intl_attrs.name_plural_id = product_name_plurals_intl.id
                
                LEFT JOIN public.ingredient_quantities AS quantities
                    ON ingredients.id = quantities.ingredient_id
                LEFT JOIN public.units AS quantity_units
                    ON quantities.unit_id = quantity_units.id
                LEFT JOIN public.units_intl_attrs AS quantity_unit_intl_attrs
                    ON quantity_units.id = quantity_unit_intl_attrs.unit_id
                LEFT JOIN public.plurals_intl AS quantity_unit_names
                    ON quantity_unit_intl_attrs.name_plural_id = quantity_unit_names.id

                JOIN public.recipes_intl_attrs AS recipes_intl_attrs
                    ON ingredients.recipe_id = recipes_intl_attrs.recipe_id
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

    let shopping_list: Vec<_> = match db_conn.query(&stmt, &[]).await {
        Ok(rows) => rows
            .iter()
            .fold(HashMap::new(), |mut acc, row| {
                let shopping_list_item_id: Uuid = row.get("id");
                let shopping_list_item =
                    acc.entry(shopping_list_item_id)
                        .or_insert(ShoppingListItem {
                            id: shopping_list_item_id,
                            in_cart: row.get("in_cart"),
                            product: Product {
                                id: row.get("product_id"),
                                name: Plural {
                                    zero: row.get("product_name_zero"),
                                    one: row.get("product_name_one"),
                                    two: row.get("product_name_two"),
                                    few: row.get("product_name_few"),
                                    many: row.get("product_name_many"),
                                    other: row.get("product_name_other"),
                                },
                            },
                            quantities: Vec::new(),
                            recipe: match row.get("recipe_slug") {
                                Some(slug) => Some(Recipe {
                                    slug,
                                    name: row.get("recipe_name"),
                                }),
                                None => None,
                            },
                        });

                if let Some(size) = row.get("quantity") {
                    shopping_list_item.quantities.push(Quantity {
                        size,
                        unit: match row.get("unit_id") {
                            Some(unit_id) => Some(Unit {
                                id: unit_id,
                                name: Plural {
                                    zero: row.get("quantity_name_zero"),
                                    one: row.get("quantity_name_one"),
                                    two: row.get("quantity_name_two"),
                                    few: row.get("quantity_name_few"),
                                    many: row.get("quantity_name_many"),
                                    other: row.get("quantity_name_other"),
                                },
                            }),
                            None => None,
                        },
                    });
                }

                acc
            })
            .into_values()
            .collect(),
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(shopping_list))
}
