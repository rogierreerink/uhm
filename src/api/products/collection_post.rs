use crate::db::products::upsert;
use crate::global::AppState;
use crate::types::payloads::{collection, resource};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Resource {
    name: String,
}

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<collection::PostRequest<Resource>>,
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

    let mut resources = Vec::new();

    tracing::debug!("inserting resources");
    for item in payload.data {
        let id = Uuid::new_v4();

        if let Err(err) = upsert::upsert(
            &transaction,
            &upsert::Resource {
                id: id.clone(),
                name: item.name.clone(),
            },
        )
        .await
        {
            tracing::error!("failed to insert resource: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        resources.push(resource::PostResponse { id })
    }

    tracing::debug!("committing database transaction");
    if let Err(err) = transaction.commit().await {
        tracing::error!("failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok((
        StatusCode::CREATED,
        Json(collection::PostResponse { data: resources }),
    ))
}
