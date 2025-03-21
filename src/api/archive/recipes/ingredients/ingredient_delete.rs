use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::instrument;
use uuid::Uuid;

use crate::global::AppState;

#[axum::debug_handler]
#[instrument(skip(state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path((recipe_slug, ingredient_id)): Path<(String, Uuid)>,
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
            SELECT ingredients.id
            FROM public.recipes_intl_attrs AS recipes_intl_attrs
                JOIN public.ingredients AS ingredients
                    ON recipes_intl_attrs.recipe_id = ingredients.recipe_id
            WHERE
                recipes_intl_attrs.slug = $1 AND
                ingredients.id = $2
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

    if let Err(err) = transaction
        .query_one(&get_stmt, &[&recipe_slug, &ingredient_id])
        .await
    {
        tracing::warn!("Product could not be found: {}", err);
        return Err(StatusCode::NOT_FOUND);
    }

    let delete_quantities_stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.ingredient_quantities
            WHERE ingredient_id = $1
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
        .execute(&delete_quantities_stmt, &[&ingredient_id])
        .await
    {
        tracing::error!("Failed to delete quantities: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let delete_product_stmt = match transaction
        .prepare_cached(
            "
            DELETE FROM public.ingredients
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
        .execute(&delete_product_stmt, &[&ingredient_id])
        .await
    {
        tracing::error!("Failed to delete ingredient: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
}
