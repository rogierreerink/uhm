use crate::db::shopping_list::upsert;
use crate::global::AppState;
use crate::types::payloads::{collection, resource};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    in_cart: Option<bool>,
    source: Source,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Source {
    Product { id: Uuid },
    Temporary { data: Temporary },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Temporary {
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
        let source_id = match item.source {
            Source::Product { id } => {
                let row_id = Uuid::new_v4();
                if let Err(err) = upsert::upsert_product_link(
                    &transaction,
                    &upsert::ProductLink {
                        id: row_id,
                        product_id: id,
                    },
                )
                .await
                {
                    tracing::error!("failed to insert resource: {}", err);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                upsert::SourceId::ProductLink(row_id)
            }
            Source::Temporary { data } => {
                let row_id = Uuid::new_v4();
                if let Err(err) = upsert::upsert_temporary(
                    &transaction,
                    &upsert::Temporary {
                        id: row_id,
                        name: data.name.clone(),
                    },
                )
                .await
                {
                    tracing::error!("failed to insert resource: {}", err);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                upsert::SourceId::Temporary(row_id)
            }
        };

        let id = Uuid::new_v4();

        if let Err(err) = upsert::upsert(
            &transaction,
            &upsert::Resource {
                id: id.clone(),
                in_cart: item.in_cart.unwrap_or(false),
                source_id,
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
