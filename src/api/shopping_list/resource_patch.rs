use crate::db::shopping_list::{delete, query, upsert};
use crate::db::DbError;
use crate::global::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    in_cart: Option<bool>,
    source: Option<Source>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Source {
    Product { id: Uuid },
    Temporary { data: Temporary },
}

#[derive(Deserialize)]
struct Temporary {
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
    let current = match query::query_one(&transaction, id).await {
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

    let (new_source_id, old_source_id) = match &payload.source {
        Some(Source::Product { id: new_id }) => {
            tracing::debug!("upserting product link");

            let product_link = match &current.source {
                query::Source::ProductLink { id, .. } => upsert::ProductLink {
                    id: id.clone(),
                    product_id: new_id.clone(),
                },
                query::Source::Temporary { .. } => upsert::ProductLink {
                    id: Uuid::new_v4(),
                    product_id: new_id.clone(),
                },
            };

            if let Err(err) = upsert::upsert_product_link(&transaction, &product_link).await {
                tracing::error!("failed to upsert product link: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            (
                Some(upsert::SourceId::ProductLink(product_link.id)),
                match &current.source {
                    query::Source::ProductLink { .. } => None,
                    query::Source::Temporary { id, .. } => {
                        Some(delete::SourceId::Temporary(id.clone()))
                    }
                },
            )
        }
        Some(Source::Temporary { data }) => {
            tracing::debug!("upserting temporary data");

            let temporary = match &current.source {
                query::Source::Temporary { id, name } => upsert::Temporary {
                    id: id.clone(),
                    name: data.name.clone().unwrap_or(name.clone()),
                },
                query::Source::ProductLink { .. } => upsert::Temporary {
                    id: Uuid::new_v4(),
                    name: match data.name.clone() {
                        Some(name) => name,
                        None => {
                            tracing::error!("Temporary item must have a name");
                            return Err(StatusCode::BAD_REQUEST);
                        }
                    },
                },
            };

            if let Err(err) = upsert::upsert_temporary(&transaction, &temporary).await {
                tracing::error!("failed to upsert temporary item: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            (
                Some(upsert::SourceId::Temporary(temporary.id)),
                match &current.source {
                    query::Source::Temporary { .. } => None,
                    query::Source::ProductLink { id, .. } => {
                        Some(delete::SourceId::ProductLink(id.clone()))
                    }
                },
            )
        }
        None => (None, None),
    };

    tracing::debug!("updating resource");
    if let Err(err) = upsert::upsert(
        &transaction,
        &upsert::Resource {
            id,
            in_cart: payload.in_cart.unwrap_or(current.in_cart),
            source_id: new_source_id.unwrap_or_else(|| match current.source {
                query::Source::ProductLink { id, .. } => upsert::SourceId::ProductLink(id.clone()),
                query::Source::Temporary { id, .. } => upsert::SourceId::Temporary(id.clone()),
            }),
        },
    )
    .await
    {
        tracing::error!("failed to update resource: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    if let Some(source_id) = old_source_id {
        tracing::debug!("cleaning up old source data");
        if let Err(err) = delete::delete_source(&transaction, &source_id).await {
            tracing::error!("failed to cleanup old source data: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    tracing::debug!("committing database transaction");
    if let Err(err) = transaction.commit().await {
        tracing::error!("failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(())
}
