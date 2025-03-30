use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::types::error::Error;

use super::DbError;

pub trait DbBlocks {
    async fn get_all(&mut self) -> Result<Vec<BlockSummary>, DbError>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Option<Block>, DbError>;
    async fn create(&mut self, block: &BlockNew) -> Result<Block, DbError>;
    async fn update(&mut self, block: &BlockUpdate) -> Result<Block, DbError>;
    async fn delete(&mut self, id: &Uuid) -> Result<(), DbError>;
}

#[derive(Serialize)]
pub struct BlockSummary {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: BlockSummaryData,
}

impl From<&Row> for BlockSummary {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

#[derive(Serialize)]
pub struct BlockSummaryData {
    pub kind: BlockSummaryKind,
}

impl From<&Row> for BlockSummaryData {
    fn from(row: &Row) -> Self {
        Self { kind: row.into() }
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockSummaryKind {
    IngredientCollection {
        id: Uuid,
        data: IngredientCollectionSummaryData,
    },
    Paragraph {
        data: ParagraphSummaryData,
    },
}

impl From<&Row> for BlockSummaryKind {
    fn from(row: &Row) -> Self {
        if let Some(_) = row.get::<_, Option<Uuid>>("ingredient_collection_block_id") {
            Self::IngredientCollection {
                id: row.get("ingredient_collection_id"),
                data: row.into(),
            }
        } else if let Some(_) = row.get::<_, Option<Uuid>>("paragraph_block_id") {
            Self::Paragraph { data: row.into() }
        } else {
            panic!()
        }
    }
}

#[derive(Serialize)]
pub struct IngredientCollectionSummaryData {}

impl From<&Row> for IngredientCollectionSummaryData {
    fn from(row: &Row) -> Self {
        Self {}
    }
}

#[derive(Serialize)]
pub struct ParagraphSummaryData {
    pub text: String,
}

impl From<&Row> for ParagraphSummaryData {
    fn from(row: &Row) -> Self {
        Self {
            text: row.get("paragraph_block_text"),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Block {
    pub id: Uuid,
    pub ts_created: DateTime<Utc>,
    pub ts_updated: Option<DateTime<Utc>>,
    pub data: BlockData,
}

impl From<&Row> for Block {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: row.into(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct BlockData {
    pub kind: BlockKind,
}

impl From<&Row> for BlockData {
    fn from(row: &Row) -> Self {
        Self { kind: row.into() }
    }
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockKind {
    IngredientCollection {
        id: Uuid,
        data: IngredientCollectionData,
    },
    Paragraph {
        data: ParagraphData,
    },
}

impl From<&Row> for BlockKind {
    fn from(row: &Row) -> Self {
        if let Some(_) = row.get::<_, Option<Uuid>>("ingredient_collection_block_id") {
            Self::IngredientCollection {
                id: row.get("ingredient_collection_id"),
                data: row.into(),
            }
        } else if let Some(_) = row.get::<_, Option<Uuid>>("paragraph_block_id") {
            Self::Paragraph { data: row.into() }
        } else {
            panic!()
        }
    }
}

#[derive(Serialize, Clone)]
pub struct IngredientCollectionData {}

impl From<&Row> for IngredientCollectionData {
    fn from(_: &Row) -> Self {
        Self {}
    }
}

#[derive(Serialize, Clone)]
pub struct ParagraphData {
    pub text: String,
}

impl From<&Row> for ParagraphData {
    fn from(row: &Row) -> Self {
        Self {
            text: row.get("paragraph_block_text"),
        }
    }
}

#[derive(Deserialize)]
pub struct BlockNew {
    pub data: BlockNewData,
}

#[derive(Deserialize)]
pub struct BlockNewData {
    pub kind: BlockNewKind,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockNewKind {
    IngredientCollection { id: Uuid },
    Paragraph { data: ParagraphNewData },
}

#[derive(Deserialize)]
pub struct ParagraphNewData {
    pub text: String,
}

#[derive(Deserialize)]
pub struct BlockUpdate {
    pub data: BlockUpdateData,
}

#[derive(Deserialize)]
pub struct BlockUpdateData {}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockUpdateKind {
    IngredientCollection { id: Uuid },
    Paragraph { data: ParagraphUpdateData },
}

#[derive(Deserialize)]
pub struct ParagraphUpdateData {
    pub text: Option<String>,
}

pub struct DbBlocksPostgres {
    connection: Object<Manager>,
}

impl DbBlocksPostgres {
    pub fn new(connection: Object<Manager>) -> Self {
        Self { connection }
    }
}

impl DbBlocks for DbBlocksPostgres {
    async fn get_all(&mut self) -> Result<Vec<BlockSummary>, DbError> {
        tracing::debug!("preparing cached statement");
        let stmt = match self
            .connection
            .prepare_cached(
                "
                SELECT
                    blocks.id,
                    blocks.ts_created,
                    blocks.ts_updated,
                    ingredient_collection_blocks.id AS ingredient_collection_block_id,
                    ingredient_collection_blocks.ingredient_collection_id AS ingredient_collection_id,
                    paragraph_blocks.id AS paragraph_block_id,
                    paragraph_blocks.text AS paragraph_block_text

                FROM public.blocks
                    LEFT JOIN public.ingredient_collection_blocks
                        ON blocks.ingredient_collection_block_id = ingredient_collection_blocks.id
                    LEFT JOIN public.paragraph_blocks
                        ON blocks.paragraph_block_id = paragraph_blocks.id

                ORDER BY blocks.id
                "
            )
            .await
        {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("failed to prepare statement: {}", err);
                return Err(DbError::Connection);
            }
        };

        tracing::debug!("querying database");
        let result = match self.connection.query(&stmt, &[]).await {
            Ok(rows) => rows.into_iter().map(|row| (&row).into()).collect(),
            Err(err) => {
                tracing::error!("failed to query database: {}", err);
                return Err(DbError::Query);
            }
        };

        Ok(result)
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Option<Block>, DbError> {
        tracing::debug!("preparing cached statement");
        let stmt = match self.connection.prepare_cached("").await {
            Ok(stmt) => stmt,
            Err(err) => {
                tracing::error!("failed to prepare statement: {}", err);
                return Err(DbError::Connection);
            }
        };

        tracing::debug!("querying database");
        let result = match self.connection.query(&stmt, &[&id]).await {
            Ok(rows) => match rows
                .into_iter()
                .map(|row| (&row).into())
                .collect::<Vec<Block>>()
            {
                rows if rows.len() >= 2 => return Err(DbError::TooMany),
                rows => rows.get(0).cloned(),
            },
            Err(err) => {
                tracing::error!("failed to query database: {}", err);
                return Err(DbError::Query);
            }
        };

        Ok(result)
    }

    async fn create(&mut self, block: &BlockNew) -> Result<Block, DbError> {
        tracing::debug!("starting database transaction");
        let transaction = match self.connection.transaction().await {
            Ok(transaction) => transaction,
            Err(err) => {
                tracing::error!("failed to start a database transaction: {}", err);
                return Err(DbError::Connection);
            }
        };

        match &block.data.kind {
            BlockNewKind::IngredientCollection { id } => {
                tracing::debug!("creating ingredient collection block: preparing cached statement");
                let stmt = match transaction.prepare_cached("").await {
                    Ok(stmt) => stmt,
                    Err(err) => {
                        tracing::error!("failed to prepare statement: {}", err);
                        return Err(DbError::Connection);
                    }
                };

                tracing::debug!("creating ingredient collection block: executing query");
                match transaction.execute(&stmt, &[]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("failed to query database: {}", err);
                        return Err(DbError::Query);
                    }
                }

                tracing::debug!("creating block: preparing cached statement");
                let stmt = match transaction.prepare_cached("").await {
                    Ok(stmt) => stmt,
                    Err(err) => {
                        tracing::error!("failed to prepare statement: {}", err);
                        return Err(DbError::Connection);
                    }
                };

                tracing::debug!("creating block: executing query");
                match transaction.execute(&stmt, &[]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("failed to query database: {}", err);
                        return Err(DbError::Query);
                    }
                }
            }
            BlockNewKind::Paragraph { data } => {
                tracing::debug!("creating paragraph block: preparing cached statement");
                let stmt = match transaction.prepare_cached("").await {
                    Ok(stmt) => stmt,
                    Err(err) => {
                        tracing::error!("failed to prepare statement: {}", err);
                        return Err(DbError::Connection);
                    }
                };

                tracing::debug!("creating paragraph block: executing query");
                match transaction.execute(&stmt, &[]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("failed to query database: {}", err);
                        return Err(DbError::Query);
                    }
                }

                tracing::debug!("creating block: preparing cached statement");
                let stmt = match transaction.prepare_cached("").await {
                    Ok(stmt) => stmt,
                    Err(err) => {
                        tracing::error!("failed to prepare statement: {}", err);
                        return Err(DbError::Connection);
                    }
                };

                tracing::debug!("creating block: executing query");
                match transaction.execute(&stmt, &[]).await {
                    Ok(_) => {}
                    Err(err) => {
                        tracing::error!("failed to query database: {}", err);
                        return Err(DbError::Query);
                    }
                }
            }
        }

        tracing::debug!("committing database transaction");
        if let Err(err) = transaction.commit().await {
            tracing::error!("failed to commit transaction: {}", err);
            return Err(DbError::Connection);
        }

        todo!()
    }

    async fn update(&mut self, block: &BlockUpdate) -> Result<Block, DbError> {
        todo!()
    }

    async fn delete(&mut self, id: &Uuid) -> Result<(), DbError> {
        todo!()
    }
}
