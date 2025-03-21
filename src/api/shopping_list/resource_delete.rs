use crate::db::shopping_list::{delete, query};
use crate::db::DbError;
use crate::global::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;
use uuid::Uuid;

#[tracing::instrument(skip(state))]
pub async fn handle(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> impl IntoResponse {
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
    let resource = match query::query_one(&transaction, id).await {
        Ok(resource) => resource,
        Err(err) if err == DbError::NotFound => {
            tracing::warn!("resource could not be found");
            return Err(StatusCode::NOT_FOUND);
        }
        Err(err) => {
            tracing::error!("failed to query resource: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    tracing::debug!("deleting resource");
    if let Err(err) = delete::delete(&transaction, &id).await {
        tracing::error!("failed to delete resource: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    tracing::debug!("deleting source data");
    let source_id = match resource.source {
        query::Source::ProductLink { id, .. } => delete::SourceId::ProductLink(id),
        query::Source::Temporary { id, .. } => delete::SourceId::Temporary(id),
    };
    if let Err(err) = delete::delete_source(&transaction, &source_id).await {
        tracing::error!("failed to delete source data: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    tracing::debug!("committing database transaction");
    if let Err(err) = transaction.commit().await {
        tracing::error!("failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
}
