use crate::db::ingredients::upsert;
use crate::db::{ingredient_collections, DbError};
use crate::global::AppState;
use crate::types::payloads::{collection, resource};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    product_link: ProductLink,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductLink {
    id: Uuid,
}

pub async fn handle(
    State(state): State<Arc<AppState>>,
    Path(collection_id): Path<Uuid>,
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

    tracing::debug!("querying parent collection");
    match ingredient_collections::query::query_one(&transaction, &collection_id).await {
        Ok(_) => {}
        Err(err) if err == DbError::NotFound => {
            tracing::error!("parent collection could not be found: {}", err);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            tracing::error!("failed to query parent collection: {}", err);
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
                ingredient_collection_id: collection_id,
                product_id: item.product_link.id,
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
