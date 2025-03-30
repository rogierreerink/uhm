use crate::db::DbError;
use crate::types::error::Error;
use chrono::Local;
use deadpool_postgres::Transaction;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub id: Uuid,
    pub kind_id: KindId,
}

#[derive(Debug, PartialEq, Clone)]
pub enum KindId {
    IngredientCollection(Uuid),
    Paragraph(Uuid),
}

pub async fn upsert_block<'a>(
    transaction: &'a Transaction<'a>,
    block: &Block,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    let (ingredient_collection_block_id, paragraph_block_id) = match &block.kind_id {
        KindId::IngredientCollection(id) => (None, Some(id.clone())),
        KindId::Paragraph(id) => (Some(id.clone()), None),
    };

    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.blocks (
                id,
                ingredient_collection_block_id,
                paragraph_block_id,
                ts_updated
            )
            VALUES ($1, $2, $3, $4, NULL)
            ON CONFLICT (id) DO UPDATE SET
                ingredient_collection_block_id = EXCLUDED.ingredient_collection_block_id,
                paragraph_block_id = EXCLUDED.paragraph_block_id,
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
                &block.id,
                &ingredient_collection_block_id,
                &paragraph_block_id,
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
pub struct IngredientCollectionBlock {
    pub id: Uuid,
    pub ingredient_collection_id: Uuid,
}

pub async fn upsert_ingredient_collection_block<'a>(
    transaction: &'a Transaction<'a>,
    ingredient_collection_block: &IngredientCollectionBlock,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.ingredient_collection_blocks (
                id,
                ingredient_collection_id,
                ts_updated
            )
            VALUES ($1, $2, NULL)
            ON CONFLICT (id) DO UPDATE SET
                ingredient_collection_id = EXCLUDED.ingredient_collection_id,
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
        .execute(
            &stmt,
            &[
                &ingredient_collection_block.id,
                &ingredient_collection_block.ingredient_collection_id,
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
pub struct ParagraphBlock {
    pub id: Uuid,
    pub text: String,
}

pub async fn upsert_paragraph_block<'a>(
    transaction: &'a Transaction<'a>,
    paragraph_block: &ParagraphBlock,
) -> Result<(), Error<DbError, tokio_postgres::Error>> {
    tracing::debug!("preparing cached statement");
    let stmt = match transaction
        .prepare_cached(
            "
            INSERT INTO public.ingredient_collection_blocks (
                id,
                text,
                ts_updated
            )
            VALUES ($1, $2, NULL)
            ON CONFLICT (id) DO UPDATE SET
                text = EXCLUDED.text,
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
        .execute(
            &stmt,
            &[&paragraph_block.id, &paragraph_block.text, &Local::now()],
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => return Err(err.into()),
    }
}
