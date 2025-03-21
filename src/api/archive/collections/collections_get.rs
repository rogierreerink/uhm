use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use tracing::instrument;

use crate::global::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CollectionLink {
    slug: String,
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
              attrs.slug AS slug,
              attrs.name AS name

            FROM public.collections AS collections
              JOIN public.collections_intl_attrs AS attrs
                ON collections.id = attrs.collection_id
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

    let collections: Vec<_> = match db_conn.query(&stmt, &[]).await {
        Ok(rows) => rows
            .into_iter()
            .map(|row| CollectionLink {
                slug: row.get("slug"),
                name: row.get("name"),
            })
            .collect(),
        Err(err) => {
            tracing::error!("Failed to execute query: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(collections))
}
