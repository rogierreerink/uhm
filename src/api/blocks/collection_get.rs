use crate::db::blocks::query;
use crate::global::AppState;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Response {
    data: Vec<Block>,
}

#[derive(Serialize)]
pub struct Block {
    id: Uuid,
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
    data: BlockData,
}

#[derive(Serialize)]
pub struct BlockData {
    kind: Kind,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Kind {
    IngredientCollection {
        id: Uuid,
        data: IngredientCollectionData,
    },
    Paragraph {
        data: ParagraphData,
    },
}

#[derive(Serialize)]
struct IngredientCollectionData {}

#[derive(Serialize)]
struct ParagraphData {
    text: String,
}

pub async fn handle(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

    tracing::debug!("querying collection");
    let data = match query::query_blocks(&transaction).await {
        Ok(items) => items
            .iter()
            .map(|item| Block {
                id: item.id,
                created: item.ts_created,
                updated: item.ts_updated,
                data: BlockData {
                    kind: match &item.kind {
                        query::Kind::IngredientCollection {
                            ingredient_collection,
                            ..
                        } => Kind::IngredientCollection {
                            id: ingredient_collection.id.clone(),
                            data: IngredientCollectionData {},
                        },
                        query::Kind::Paragraph { text, .. } => Kind::Paragraph {
                            data: ParagraphData { text: text.clone() },
                        },
                    },
                },
            })
            .collect(),
        Err(err) => {
            tracing::error!("failed to query collection: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(Json(Response { data }))
}
