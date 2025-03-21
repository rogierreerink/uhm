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

use crate::global::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Recipe {
    slug: String,
    name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Collection {
    slug: String,
    name: String,
    recipes: Vec<Recipe>,
}

#[instrument(skip(state))]
pub async fn handle(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("New request");

    let db_conn = match state.db_pool.get().await {
        Ok(db_conn) => db_conn,
        Err(err) => {
            tracing::error!("Failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let collection_stmt = match db_conn
        .prepare_cached(
            "
            SELECT
                intl_attrs.collection_id AS id,
                intl_attrs.slug AS slug,
                intl_attrs.name AS name
            FROM public.collections_intl_attrs AS intl_attrs
            WHERE intl_attrs.slug = $1
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

    let (id, slug, name) = match db_conn.query_one(&collection_stmt, &[&slug]).await {
        Ok(row) => (
            row.get::<_, Uuid>("id"),
            row.get::<_, String>("slug"),
            row.get::<_, String>("name"),
        ),
        Err(err) => {
            tracing::warn!("Collection could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let recipes_stmt = match db_conn
        .prepare_cached(
            "
            SELECT
                recipe_intl_attrs.slug AS recipe_slug,
                recipe_intl_attrs.name AS recipe_name

            FROM public.collection_recipes AS collection_recipes
                JOIN public.recipes_intl_attrs AS recipe_intl_attrs
                    ON collection_recipes.recipe_id = recipe_intl_attrs.recipe_id

            WHERE collection_recipes.collection_id = $1
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

    let recipes = match db_conn.query(&recipes_stmt, &[&id]).await {
        Ok(rows) => rows
            .iter()
            .map(|row| Recipe {
                slug: row.get("recipe_slug"),
                name: row.get("recipe_name"),
            })
            .collect(),
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let collection = Collection {
        slug,
        name,
        recipes,
    };

    Ok(Json(collection))
}
