use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{api::plurals::Plural, global::AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    id: Uuid,
    name: Plural,
}

#[derive(Deserialize, Debug)]
pub struct Search {
    name: Option<String>,
}

#[instrument(skip(state))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Query(query): Query<Search>,
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
                intl_attrs.product_id AS id,
                name_plurals.zero AS name_zero,
                name_plurals.one AS name_one,
                name_plurals.two AS name_two,
                name_plurals.few AS name_few,
                name_plurals.many AS name_many,
                name_plurals.other AS name_other
                
            FROM public.products_intl_attrs AS intl_attrs
                JOIN public.plurals_intl AS name_plurals
                    ON intl_attrs.name_plural_id = name_plurals.id

            WHERE
                -- Search by name
                ($1::varchar IS NULL OR (
                    LOWER(name_plurals.zero) LIKE '%'||LOWER($1)||'%' OR
                    LOWER(name_plurals.one) LIKE '%'||LOWER($1)||'%' OR
                    LOWER(name_plurals.two) LIKE '%'||LOWER($1)||'%' OR
                    LOWER(name_plurals.few) LIKE '%'||LOWER($1)||'%' OR
                    LOWER(name_plurals.many) LIKE '%'||LOWER($1)||'%' OR
                    LOWER(name_plurals.other) LIKE '%'||LOWER($1)||'%'
                ))
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

    let products: Vec<_> = match db_conn.query(&stmt, &[&query.name]).await {
        Ok(rows) => rows
            .iter()
            .map(|row| Product {
                id: row.get("id"),
                name: Plural {
                    zero: row.get("name_zero"),
                    one: row.get("name_one"),
                    two: row.get("name_two"),
                    few: row.get("name_few"),
                    many: row.get("name_many"),
                    other: row.get("name_other"),
                },
            })
            .collect(),
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(products))
}
