use crate::db::products::{query, upsert};
use crate::db::DbError;
use crate::global::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Resource {
    name: Option<String>,
}

#[tracing::instrument(skip(state, payload))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Resource>,
) -> impl IntoResponse {
    tracing::debug!("setting up database connection");
    let mut connection = match state.db_pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("failed to get a database connection from the pool: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("starting database transaction");
    let transaction = match connection.transaction().await {
        Ok(transaction) => transaction,
        Err(err) => {
            tracing::error!("failed to start a database transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("querying resource");
    let current = match query::query_one(&transaction, &id).await {
        Ok(item) => item,
        Err(err) if err == DbError::NotFound => {
            tracing::warn!("resource could not be found");
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            tracing::error!("failed to query resource: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("updating resource");
    if let Err(err) = upsert::upsert(
        &transaction,
        &upsert::Resource {
            id: current.id.clone(),
            name: payload.name.unwrap_or(current.name.clone()),
        },
    )
    .await
    {
        tracing::error!("failed to update resource: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    tracing::debug!("committing database transaction");
    if let Err(err) = transaction.commit().await {
        tracing::error!("failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
}
