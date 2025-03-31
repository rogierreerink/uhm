use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;

use super::DbError;

#[trait_variant::make(Send)]
pub trait DbBlocks {
    async fn get(&mut self) -> Result<Vec<BlockSummary>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Block>;
    async fn create(&mut self, blocks: &Vec<BlockNew>) -> Result<Vec<Block>>;
    async fn update(&mut self, id: &Uuid, block: &BlockUpdate) -> Result<Block>;
    async fn delete(&mut self, id: &Uuid) -> Result<()>;
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
    fn from(_: &Row) -> Self {
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
    pub kind: Option<BlockUpdateKind>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockUpdateKind {
    IngredientCollection { id: Option<Uuid> },
    Paragraph { data: Option<ParagraphUpdateData> },
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
    async fn get(&mut self) -> Result<Vec<BlockSummary>> {
        tracing::debug!("preparing cached statement");
        let stmt = self
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
            .await?;

        tracing::debug!("executing query");
        let result = self
            .connection
            .query(&stmt, &[])
            .await?
            .into_iter()
            .map(|row| (&row).into())
            .collect();

        Ok(result)
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Block> {
        tracing::debug!("preparing cached statement");
        let stmt = self
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

                WHERE blocks.id = $1
                ",
            )
            .await?;

        tracing::debug!("executing query");
        let result = match self.connection.query(&stmt, &[&id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            rows => (&rows[0]).into(),
        };

        Ok(result)
    }

    async fn create(&mut self, blocks: &Vec<BlockNew>) -> Result<Vec<Block>> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        let mut inserted = Vec::new();

        for block in blocks {
            let block_id = Uuid::new_v4();
            let block_kind_id = Uuid::new_v4();

            inserted.push(match &block.kind {
                BlockNewKind::IngredientCollection { id } => {
                    tracing::debug!(
                        "create ingredient collection block: preparing cached statement"
                    );
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.ingredient_collection_blocks (
                                id,
                                ingredient_collection_id
                            )
                            VALUES (
                                $1, $2
                            )
                            ",
                        )
                        .await?;

                    tracing::debug!("create ingredient collection block: executing query");
                    transaction.execute(&stmt, &[&block_kind_id, &id]).await?;

                    tracing::debug!("create block: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.blocks (
                                id,
                                ingredient_collection_block_id
                            )
                            VALUES (
                                $1, $2
                            )
                            RETURNING ts_created
                            ",
                        )
                        .await?;

                    tracing::debug!("create block: executing query");
                    let row = transaction
                        .query_one(&stmt, &[&block_id, &block_kind_id])
                        .await?;

                    Block {
                        id: block_id,
                        ts_created: row.get("ts_created"),
                        ts_updated: None,
                        data: BlockData {
                            kind: BlockKind::IngredientCollection {
                                id: *id,
                                data: IngredientCollectionData {},
                            },
                        },
                    }
                }
                BlockNewKind::Paragraph { data } => {
                    tracing::debug!("create paragraph block: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.paragraph_blocks (
                                id,
                                text
                            )
                            VALUES (
                                $1, $2
                            )
                            ",
                        )
                        .await?;

                    tracing::debug!("create paragraph block: executing query");
                    transaction
                        .execute(&stmt, &[&block_kind_id, &data.text])
                        .await?;

                    tracing::debug!("create block: preparing cached statement");
                    let stmt = transaction
                        .prepare_cached(
                            "
                            INSERT INTO public.blocks (
                                id,
                                paragraph_block_id
                            )
                            VALUES (
                                $1, $2
                            )
                            RETURNING ts_created
                            ",
                        )
                        .await?;

                    tracing::debug!("create block: executing query");
                    let row = transaction
                        .query_one(&stmt, &[&block_id, &block_kind_id])
                        .await?;

                    Block {
                        id: block_id,
                        ts_created: row.get("ts_created"),
                        ts_updated: None,
                        data: BlockData {
                            kind: BlockKind::Paragraph {
                                data: ParagraphData {
                                    text: data.text.clone(),
                                },
                            },
                        },
                    }
                }
            })
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(inserted)
    }

    async fn update(&mut self, id: &Uuid, block: &BlockUpdate) -> Result<Block> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("select current block: preparing cached statement");
        let stmt = transaction
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

                WHERE blocks.id = $1
                ",
            )
            .await?;

        tracing::debug!("select current block: executing query");
        let current = match transaction.query(&stmt, &[&id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            rows => rows[0].clone(),
        };

        let updated_block_kind = match &block.kind {
            Some(BlockUpdateKind::IngredientCollection { id }) => {
                let block_id: Uuid = match current.get("ingredient_collection_block_id") {
                    Some(id) => id,
                    None => return Err(DbError::InvalidOperation.into()),
                };

                tracing::debug!("update ingredient collection block: executing query");
                let stmt = transaction
                    .prepare_cached(
                        "
                        UPDATE public.ingredient_collection_blocks
                        SET ingredient_collection_id = $2,
                            ts_updated = CURRENT_TIMESTAMP
                        WHERE id = $1
                        ",
                    )
                    .await?;

                let ingredient_collection_id =
                    id.unwrap_or(current.get("ingredient_collection_id"));

                tracing::debug!("update ingredient collection block: executing query");
                transaction
                    .execute(&stmt, &[&block_id, &ingredient_collection_id])
                    .await?;

                BlockKind::IngredientCollection {
                    id: ingredient_collection_id,
                    data: IngredientCollectionData {},
                }
            }

            Some(BlockUpdateKind::Paragraph { data }) => {
                let block_id: Uuid = match current.get("paragraph_block_id") {
                    Some(id) => id,
                    None => return Err(DbError::InvalidOperation.into()),
                };

                tracing::debug!("update paragraph block: executing query");
                let stmt = transaction
                    .prepare_cached(
                        "
                        UPDATE public.paragraph_blocks
                        SET text = $2,
                            ts_updated = CURRENT_TIMESTAMP
                        WHERE id = $1
                        ",
                    )
                    .await?;

                let text = data
                    .as_ref()
                    .and_then(|data| data.text.clone())
                    .unwrap_or(current.get("paragraph_block_text"));

                tracing::debug!("update paragraph block: executing query");
                transaction.execute(&stmt, &[&block_id, &text]).await?;

                BlockKind::Paragraph {
                    data: ParagraphData { text },
                }
            }

            None => Into::<BlockKind>::into(&current),
        };

        tracing::debug!("update block: executing query");
        let stmt = transaction
            .prepare_cached(
                "
                UPDATE public.blocks
                SET ts_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                ",
            )
            .await?;

        tracing::debug!("update block: executing query");
        transaction.execute(&stmt, &[&id]).await?;

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(Block {
            id: current.get("id"),
            ts_created: current.get("ts_created"),
            ts_updated: current.get("ts_updated"),
            data: BlockData {
                kind: updated_block_kind,
            },
        })
    }

    async fn delete(&mut self, id: &Uuid) -> Result<()> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("delete block: preparing cached statement");
        let stmt = transaction
            .prepare_cached(
                "
                DELETE FROM public.blocks
                WHERE blocks.id = $1
                RETURNING
                    ingredient_collection_block_id,
                    paragraph_block_id
                ",
            )
            .await?;

        tracing::debug!("delete block: executing query");
        let block = match transaction.query(&stmt, &[&id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            rows => rows[0].clone(),
        };

        if let Some(block_id) = block.get::<_, Option<Uuid>>("ingredient_collection_block_id") {
            tracing::debug!("delete ingredient collection block: preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    DELETE FROM public.ingredient_collection_blocks
                    WHERE id = $1
                    ",
                )
                .await?;

            tracing::debug!("delete ingredient collection block: executing query");
            match transaction.execute(&stmt, &[&block_id]).await? {
                count if count == 0 => return Err(DbError::NotFound.into()),
                count if count >= 2 => return Err(DbError::TooMany.into()),
                _ => (),
            };
        }

        if let Some(block_id) = block.get::<_, Option<Uuid>>("paragraph_block_id") {
            tracing::debug!("delete paragraph block: preparing cached statement");
            let stmt = transaction
                .prepare_cached(
                    "
                    DELETE FROM public.paragraph_blocks
                    WHERE id = $1
                    ",
                )
                .await?;

            tracing::debug!("delete paragraph block: executing query");
            match transaction.execute(&stmt, &[&block_id]).await? {
                count if count == 0 => return Err(DbError::NotFound.into()),
                count if count >= 2 => return Err(DbError::TooMany.into()),
                _ => (),
            };
        }

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(())
    }
}
