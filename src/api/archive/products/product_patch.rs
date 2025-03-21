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

use crate::{api::plurals::Plural, global::AppState};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    id: Uuid,
    name: Plural,
}

#[axum::debug_handler]
#[instrument(skip(state, payload))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
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
                intl_attrs.product_id AS id,
                name_plurals.id as name_plural_id,
                name_plurals.zero AS name_zero,
                name_plurals.one AS name_one,
                name_plurals.two AS name_two,
                name_plurals.few AS name_few,
                name_plurals.many AS name_many,
                name_plurals.other AS name_other

            FROM public.products_intl_attrs AS intl_attrs
                JOIN public.plurals_intl AS name_plurals
                    ON intl_attrs.name_plural_id = name_plurals.id

            WHERE intl_attrs.product_id = $1
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

    let get_query_result = match transaction.query_one(&get_stmt, &[&id]).await {
        Ok(row) => row,
        Err(err) => {
            tracing::warn!("Product could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let current_state = Product {
        id,
        name: Plural {
            zero: get_query_result.get("name_zero"),
            one: get_query_result.get("name_one"),
            two: get_query_result.get("name_two"),
            few: get_query_result.get("name_few"),
            many: get_query_result.get("name_many"),
            other: get_query_result.get("name_other"),
        },
    };

    let mut state_json = match serde_json::to_value(&current_state) {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("Failed to serialize product: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if let Err(err) = json_patch::patch(&mut state_json, &payload) {
        tracing::error!("Failed to patch product: {}", err);
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_state = match serde_json::from_value::<Product>(state_json) {
        Ok(state) if state.id != current_state.id => {
            tracing::warn!("Client tried to alter the product ID");
            return Err(StatusCode::BAD_REQUEST);
        }
        Ok(state) => state,
        Err(err) => {
            tracing::warn!("Failed to deserialize patch result: {}", err);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if new_state.name != current_state.name {
        tracing::info!("Patching product name");

        let update_name_plural_stmt = match transaction
            .prepare_cached(
                "
                UPDATE public.plurals_intl
                SET zero = $2,
                    one = $3,
                    two = $4,
                    few = $5,
                    many = $6,
                    other = $7
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
                &update_name_plural_stmt,
                &[
                    &get_query_result.get::<_, Uuid>("name_plural_id"),
                    &new_state.name.zero,
                    &new_state.name.one,
                    &new_state.name.two,
                    &new_state.name.few,
                    &new_state.name.many,
                    &new_state.name.other,
                ],
            )
            .await
        {
            Ok(update_count) if update_count == 0 => {
                tracing::error!("Product name was not updated");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(_) => (),
            Err(err) => {
                tracing::error!("Product name could not be updated: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(new_state))
}
