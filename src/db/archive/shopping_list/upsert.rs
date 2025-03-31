use crate::db::DbError;
use crate::types::error::Error;
use chrono::Local;
use deadpool_postgres::Transaction;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Resource {
    pub id: Uuid,
    pub in_cart: bool,
    pub source_id: SourceId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SourceId {
    ProductLink(Uuid),
    Temporary(Uuid),
}

pub async fn upsert<'a>(
    transaction: &'a Transaction<'a>,
    resource: &Resource,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    let (temp_item_id, product_link_id) = match &resource.source_id {
        SourceId::ProductLink(id) => (None, Some(id.clone())),
        SourceId::Temporary(id) => (Some(id.clone()), None),
    };

    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.shopping_list (id, in_cart, temp_item_id, product_link_id, ts_updated)
            VALUES ($1, $2, $3, $4, NULL)
            ON CONFLICT (id) DO UPDATE SET
                in_cart = EXCLUDED.in_cart,
                temp_item_id = EXCLUDED.temp_item_id,
                product_link_id = EXCLUDED.product_link_id,
                ts_updated = $5
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction
        .execute(
            &stmt,
            &[
                &resource.id,
                &resource.in_cart,
                &temp_item_id,
                &product_link_id,
                &Local::now(),
            ],
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Temporary {
    pub id: Uuid,
    pub name: String,
}

pub async fn upsert_temporary<'a>(
    transaction: &'a Transaction<'a>,
    resource: &Temporary,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.shopping_list_temp_items (id, name, ts_updated)
            VALUES ($1, $2, NULL)
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                ts_updated = $3
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction
        .execute(&stmt, &[&resource.id, &resource.name, &Local::now()])
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProductLink {
    pub id: Uuid,
    pub product_id: Uuid,
}

pub async fn upsert_product_link<'a>(
    transaction: &'a Transaction<'a>,
    resource: &ProductLink,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.shopping_list_product_links (id, product_id, ts_updated)
            VALUES ($1, $2, NULL)
            ON CONFLICT (id) DO UPDATE SET
                product_id = EXCLUDED.product_id,
                ts_updated = $3
            ",
        )
        .await
    {
        Ok(stmt) => stmt,
        Err(err) => return Err(err.into()),
    };

    tracing::debug!("executing query");
    match transaction
        .execute(&stmt, &[&resource.id, &resource.product_id, &Local::now()])
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}
