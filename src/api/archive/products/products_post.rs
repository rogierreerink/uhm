use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{api::plurals::Plural, global::AppState};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRequest {
    name: Plural,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductResponse {
    id: Uuid,
    name: Plural,
}

#[axum::debug_handler]
#[instrument(skip(state, products))]
pub async fn handle(
    State(state): State<Arc<AppState>>,
    Json(products): Json<Vec<ProductRequest>>,
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
            tracing::error!("Failed to prepare transaction: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let mut inserted_products = Vec::new();
    for product in products {
        let insert_name_plural_stmt = match transaction
            .prepare_cached(
                "
                INSERT INTO public.plurals_intl (
                    id, zero, one, two, few, many, other
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7
                )
                ",
            )
            .await
        {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("Failed to prepare statement: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let name_plural_id = Uuid::new_v4();
        if let Err(err) = transaction
            .execute(
                &insert_name_plural_stmt,
                &[
                    &name_plural_id,
                    &product.name.zero,
                    &product.name.one,
                    &product.name.two,
                    &product.name.few,
                    &product.name.many,
                    &product.name.other,
                ],
            )
            .await
        {
            tracing::error!("Failed to insert plural name: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let insert_product_stmt = match transaction
            .prepare_cached(
                "
                INSERT INTO public.products (
                    id
                ) VALUES (
                    $1
                )
                ",
            )
            .await
        {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("Failed to prepare statement: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let product_id = Uuid::new_v4();
        if let Err(err) = transaction
            .execute(&insert_product_stmt, &[&product_id])
            .await
        {
            tracing::error!("Failed to insert product: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        let insert_intl_attrs_stmt = match transaction
            .prepare_cached(
                "
                INSERT INTO public.products_intl_attrs (
                    id, product_id, name_plural_id
                ) VALUES (
                    $1, $2, $3
                )
                ",
            )
            .await
        {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("Failed to prepare statement: {}", err);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        let attrs_id = Uuid::new_v4();
        if let Err(err) = transaction
            .execute(
                &insert_intl_attrs_stmt,
                &[&attrs_id, &product_id, &name_plural_id],
            )
            .await
        {
            tracing::error!("Failed to insert intl attrs: {}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        inserted_products.push(ProductResponse {
            id: product_id,
            name: product.name,
        });
    }

    if let Err(err) = transaction.commit().await {
        tracing::warn!("Failed to commit transaction: {}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(inserted_products))
}
