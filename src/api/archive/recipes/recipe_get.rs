use std::{collections::HashMap, sync::Arc};

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Collection {
    slug: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Recipe {
    slug: String,
    name: String,
    ingredients: Vec<Ingredient>,
    collections: Vec<Collection>,
}

#[instrument(skip(state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
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
            SELECT recipe_id AS id, slug, name
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

    let (id, slug, name) = match transaction.query_one(&get_recipe_stmt, &[&slug]).await {
        Ok(row) => (
            row.get::<_, Uuid>("id"),
            row.get::<_, String>("slug"),
            row.get::<_, String>("name"),
        ),
        Err(err) => {
            tracing::warn!("Recipe could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let ingredients_stmt = match transaction
        .prepare_cached(
            "
            SELECT
                ingredients.id AS id,
                products.id AS product_id,
                product_names.zero AS product_name_zero,
                product_names.one AS product_name_one,
                product_names.two AS product_name_two,
                product_names.few AS product_name_few,
                product_names.many AS product_name_many,
                product_names.other AS product_name_other,
                quantities.quantity AS quantity,
                quantity_units.id AS unit_id,
                quantity_unit_names.zero AS quantity_unit_name_zero,
                quantity_unit_names.one AS quantity_unit_name_one,
                quantity_unit_names.two AS quantity_unit_name_two,
                quantity_unit_names.few AS quantity_unit_name_few,
                quantity_unit_names.many AS quantity_unit_name_many,
                quantity_unit_names.other AS quantity_unit_name_other

            FROM public.ingredients AS ingredients
                JOIN public.products AS products
                    ON ingredients.product_id = products.id
                JOIN public.products_intl_attrs AS product_intl_attrs
                    ON products.id = product_intl_attrs.product_id
                JOIN public.plurals_intl AS product_names
                    ON product_intl_attrs.name_plural_id = product_names.id

                LEFT JOIN public.ingredient_quantities AS quantities
                    ON ingredients.id = quantities.ingredient_id
                LEFT JOIN public.units AS quantity_units
                    ON quantities.unit_id = quantity_units.id
                LEFT JOIN public.units_intl_attrs AS quantity_unit_intl_attrs
                    ON quantity_units.id = quantity_unit_intl_attrs.unit_id
                LEFT JOIN public.plurals_intl AS quantity_unit_names
                    ON quantity_unit_intl_attrs.name_plural_id = quantity_unit_names.id

            WHERE ingredients.recipe_id = $1
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

    let ingredients = match transaction.query(&ingredients_stmt, &[&id]).await {
        Ok(rows) => rows
            .iter()
            .fold(HashMap::new(), |mut acc, row| {
                let ingredient_id: Uuid = row.get("id");
                let ingredient = acc.entry(ingredient_id).or_insert(Ingredient {
                    id: ingredient_id,
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
                });

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

                acc
            })
            .into_values()
            .collect(),
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let collections_stmt = match transaction
        .prepare_cached(
            "
            SELECT
                collections_intl_attrs.slug AS slug,
                collections_intl_attrs.name AS name

            FROM public.collection_recipes AS collection_recipes
                JOIN public.collections_intl_attrs AS collections_intl_attrs
                    ON collection_recipes.collection_id = collections_intl_attrs.collection_id

            WHERE collection_recipes.recipe_id = $1
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

    let collections = match transaction.query(&collections_stmt, &[&id]).await {
        Ok(rows) => rows
            .into_iter()
            .map(|row| Collection {
                slug: row.get("slug"),
                name: row.get("name"),
            })
            .collect(),
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let recipe = Recipe {
        slug,
        name,
        ingredients,
        collections,
    };

    Ok(Json(recipe))
}
