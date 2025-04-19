use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool::managed::Object;
use deadpool_postgres::Manager;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use uuid::Uuid;
use variants::variants;

use super::DbError;

#[trait_variant::make(Send)]
pub trait DbBlocks {
    async fn get(&mut self) -> Result<Vec<BlockMinimal>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Block>;
    async fn create(&mut self, blocks: &Vec<BlockDataNew>) -> Result<Vec<BlockMinimal>>;
    async fn update(&mut self, id: &Uuid, block: &BlockDataUpdate) -> Result<BlockMinimal>;
    async fn delete(&mut self, id: &Uuid) -> Result<()>;
}

#[variants(Minimal)]
#[derive(Serialize)]
pub struct Block {
    #[variants(include(Minimal))]
    pub id: Uuid,

    #[variants(include(Minimal))]
    pub ts_created: DateTime<Utc>,

    #[variants(include(Minimal))]
    pub ts_updated: Option<DateTime<Utc>>,

    #[variants(include(Minimal), retype = "{t}{v}")]
    pub data: BlockData,
}

#[variants(Minimal)]
impl From<&Row> for Block {
    fn from(row: &Row) -> Self {
        Self {
            #[variants(include(Minimal))]
            id: row.get("id"),

            #[variants(include(Minimal))]
            ts_created: row.get("ts_created"),

            #[variants(include(Minimal))]
            ts_updated: row.get("ts_updated"),

            #[variants(include(Minimal))]
            data: row.into(),
        }
    }
}

#[variants(Minimal, New, Update)]
#[derive(Serialize, Deserialize)]
pub struct BlockData {
    #[variants(include(Minimal, New), retype = "{t}{v}")]
    #[variants(include(Update), retype = "Option<{t}{v}>")]
    pub kind: BlockKind,
}

#[variants(Minimal)]
impl From<&Row> for BlockData {
    fn from(row: &Row) -> Self {
        Self {
            #[variants(include(Minimal))]
            kind: row.into(),
        }
    }
}

#[variants(Minimal, New, Update)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockKind {
    IngredientCollection {
        #[variants(include(Minimal, New))]
        #[variants(include(Update), retype = "Option<{t}>")]
        id: Uuid,
        data: IngredientCollectionData,
    },
    Paragraph {
        #[variants(include(Minimal, New), retype = "{t}{v}")]
        #[variants(include(Update), retype = "Option<{t}{v}>")]
        data: ParagraphData,
    },
}

#[variants(Minimal)]
impl From<&Row> for BlockKind {
    fn from(row: &Row) -> Self {
        if let Some(_) = row.get::<_, Option<Uuid>>("ingredient_collection_block_id") {
            Self::IngredientCollection {
                #[variants(include(Minimal))]
                id: row.get("ingredient_collection_id"),
                data: row.into(),
            }
        } else if let Some(_) = row.get::<_, Option<Uuid>>("paragraph_block_id") {
            Self::Paragraph {
                #[variants(include(Minimal))]
                data: row.into(),
            }
        } else {
            panic!()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IngredientCollectionData {}

impl From<&Row> for IngredientCollectionData {
    fn from(_: &Row) -> Self {
        Self {}
    }
}

#[variants(Minimal, New, Update)]
#[derive(Serialize, Deserialize)]
pub struct ParagraphData {
    #[variants(include(Minimal, New))]
    #[variants(include(Update), retype = "Option<{t}>")]
    pub text: String,
}

#[variants(Minimal)]
impl From<&Row> for ParagraphData {
    fn from(row: &Row) -> Self {
        Self {
            #[variants(include(Minimal))]
            text: row.get("paragraph_block_text"),
        }
    }
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
    async fn get(&mut self) -> Result<Vec<BlockMinimal>> {
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
        let blocks = self
            .connection
            .query(&stmt, &[])
            .await?
            .into_iter()
            .map(|row| (&row).into())
            .collect();

        Ok(blocks)
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
        let block = match self.connection.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            rows => (&rows[0]).into(),
        };

        Ok(block)
    }

    async fn create(&mut self, blocks: &Vec<BlockDataNew>) -> Result<Vec<BlockMinimal>> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        let mut inserted = Vec::new();
        for block in blocks {
            let block_id = Uuid::new_v4();
            let kind_id = Uuid::new_v4();

            inserted.push(match &block.kind {
                BlockKindNew::IngredientCollection { id } => {
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
                    transaction.execute(&stmt, &[&kind_id, id]).await?;

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
                    let row = transaction.query_one(&stmt, &[&block_id, &kind_id]).await?;

                    BlockMinimal {
                        id: block_id,
                        ts_created: row.get("ts_created"),
                        ts_updated: None,
                        data: BlockDataMinimal {
                            kind: BlockKindMinimal::IngredientCollection { id: *id },
                        },
                    }
                }
                BlockKindNew::Paragraph { data } => {
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
                    transaction.execute(&stmt, &[&kind_id, &data.text]).await?;

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
                    let row = transaction.query_one(&stmt, &[&block_id, &kind_id]).await?;

                    BlockMinimal {
                        id: block_id,
                        ts_created: row.get("ts_created"),
                        ts_updated: None,
                        data: BlockDataMinimal {
                            kind: BlockKindMinimal::Paragraph {
                                data: ParagraphDataMinimal {
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

    async fn update(&mut self, id: &Uuid, block: &BlockDataUpdate) -> Result<BlockMinimal> {
        tracing::debug!("starting database transaction");
        let transaction = self.connection.transaction().await?;

        tracing::debug!("get current block: preparing cached statement");
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

        tracing::debug!("get current block: executing query");
        let current = match transaction.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        let updated_block_kind = match &block.kind {
            Some(BlockKindUpdate::IngredientCollection { id }) => {
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
                match transaction
                    .execute(&stmt, &[&block_id, &ingredient_collection_id])
                    .await?
                {
                    count if count == 0 => return Err(DbError::NotFound.into()),
                    count if count >= 2 => return Err(DbError::TooMany.into()),
                    _ => (),
                }

                BlockKindMinimal::IngredientCollection {
                    id: ingredient_collection_id,
                }
            }

            Some(BlockKindUpdate::Paragraph { data }) => {
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
                match transaction.execute(&stmt, &[&block_id, &text]).await? {
                    count if count == 0 => return Err(DbError::NotFound.into()),
                    count if count >= 2 => return Err(DbError::TooMany.into()),
                    _ => (),
                }

                BlockKindMinimal::Paragraph {
                    data: ParagraphDataMinimal { text },
                }
            }

            None => BlockKindMinimal::from(&current),
        };

        tracing::debug!("update block: executing query");
        let stmt = transaction
            .prepare_cached(
                "
                UPDATE public.blocks
                SET ts_updated = CURRENT_TIMESTAMP
                WHERE id = $1
                RETURNING ts_updated
                ",
            )
            .await?;

        tracing::debug!("update block: executing query");
        let updated = match transaction.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
        };

        tracing::debug!("committing database transaction");
        transaction.commit().await?;

        Ok(BlockMinimal {
            id: *id,
            ts_created: current.get("ts_created"),
            ts_updated: updated.get("ts_updated"),
            data: BlockDataMinimal {
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
                WHERE id = $1
                RETURNING
                    ingredient_collection_block_id,
                    paragraph_block_id
                ",
            )
            .await?;

        tracing::debug!("delete block: executing query");
        let block = match transaction.query(&stmt, &[id]).await? {
            rows if rows.len() == 0 => return Err(DbError::NotFound.into()),
            rows if rows.len() >= 2 => return Err(DbError::TooMany.into()),
            mut rows => rows.pop().unwrap(),
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
