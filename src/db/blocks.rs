use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{TryFutureExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::utilities::modifier::{Create, Modifier, Query, Reference, Update};

use super::{
    ingredient_collections::{IngredientCollectionDataTemplate, IngredientCollectionReference},
    markdown::{MarkdownDataTemplate, MarkdownReference},
    DbError,
};

#[trait_variant::make(Send)]
pub trait BlockDb {
    async fn get_multiple(&mut self) -> Result<Vec<Block>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Block>;
    async fn create_multiple(&mut self, items: Vec<BlockCreate>) -> Result<Vec<Block>>;
    async fn update_by_id(&mut self, id: &Uuid, item: BlockUpdate) -> Result<Block>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type Block = BlockTemplate<Query>;
pub type BlockCreate = BlockDataTemplate<Create>;
pub type BlockUpdate = BlockDataTemplate<Update>;
pub type BlockReference = BlockTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BlockTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<BlockDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BlockDataTemplate<M: Modifier> {
    pub kind: M::Data<BlockKindTemplate<M>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockKindTemplate<M: Modifier> {
    IngredientCollection {
        #[serde(skip)]
        link_id: M::Meta<Uuid>,
        #[serde(flatten)]
        ingredient_collection: M::Data<IngredientCollectionReference>,
    },
    Markdown {
        #[serde(skip)]
        link_id: M::Meta<Uuid>,
        #[serde(flatten)]
        markdown: M::Data<MarkdownReference>,
    },
}

impl FromRow<'_, PgRow> for Block {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: BlockDataTemplate {
                kind: {
                    if let Some(id) = row.get("ingredient_collection_block_id") {
                        BlockKindTemplate::IngredientCollection {
                            link_id: id,
                            ingredient_collection: IngredientCollectionReference {
                                id: row.get("ingredient_collection_id"),
                                data: Some(IngredientCollectionDataTemplate {
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        }
                    } else if let Some(id) = row.get("markdown_block_id") {
                        BlockKindTemplate::Markdown {
                            link_id: id,
                            markdown: MarkdownReference {
                                id: row.get("markdown_id"),
                                data: Some(MarkdownDataTemplate {
                                    markdown: row.get("markdown"),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            },
                        }
                    } else {
                        panic!("unreachable!")
                    }
                },
            },
        })
    }
}

pub struct BlockDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> BlockDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl BlockDb for BlockDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<Block>> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as(
            "
            SELECT
                blocks.id,
                blocks.ts_created,
                blocks.ts_updated,
                ingredient_collection_blocks.id AS ingredient_collection_block_id,
                ingredient_collections.id AS ingredient_collection_id,
                markdown_blocks.id AS markdown_block_id,
                markdown.id AS markdown_id,
                markdown.markdown AS markdown

            FROM public.blocks
                LEFT JOIN public.ingredient_collection_blocks
                    ON blocks.ingredient_collection_block_id = ingredient_collection_blocks.id
                LEFT JOIN public.ingredient_collections
                    ON ingredient_collection_blocks.ingredient_collection_id = ingredient_collections.id

                LEFT JOIN public.markdown_blocks
                    ON blocks.markdown_block_id = markdown_blocks.id
                LEFT JOIN public.markdown
                    ON markdown_blocks.markdown_id = markdown.id

            ORDER BY blocks.id
            ",
        )
        .fetch(&mut *conn)
        .try_collect()
        .map_err(|error| error.into())
        .await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Block> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<BlockCreate>) -> Result<Vec<Block>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match Self::create(&mut tx, item).await {
                Ok(item) => created.push(item),
                Err(error) => {
                    tx.rollback().await?;
                    return Err(error.into());
                }
            }
        }

        tx.commit().await?;

        Ok(created)
    }

    async fn update_by_id(&mut self, id: &Uuid, item: BlockUpdate) -> Result<Block> {
        let mut tx = self.pool.begin().await?;

        let updated = match Self::update_by_id(&mut tx, id, item).await {
            Ok(item) => item,
            Err(error) => {
                tx.rollback().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(updated)
    }

    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        match Self::delete_by_id(&mut tx, id).await {
            Ok(item) => item,
            Err(error) => {
                tx.rollback().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(())
    }
}

impl BlockDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<Block>
    where
        E: PgExecutor<'c>,
    {
        sqlx::query_as(
            "
            SELECT
                blocks.id,
                blocks.ts_created,
                blocks.ts_updated,
                ingredient_collection_blocks.id AS ingredient_collection_block_id,
                ingredient_collections.id AS ingredient_collection_id,
                markdown_blocks.id AS markdown_block_id,
                markdown.id AS markdown_id,
                markdown.markdown AS markdown

            FROM public.blocks
                LEFT JOIN public.ingredient_collection_blocks
                    ON blocks.ingredient_collection_block_id = ingredient_collection_blocks.id
                LEFT JOIN public.ingredient_collections
                    ON ingredient_collection_blocks.ingredient_collection_id = ingredient_collections.id

                LEFT JOIN public.markdown_blocks
                    ON blocks.markdown_block_id = markdown_blocks.id
                LEFT JOIN public.markdown
                    ON markdown_blocks.markdown_id = markdown.id

            WHERE blocks.id = $1
            ",
        )
        .bind(id)
        .fetch_one(executor)
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => Into::<anyhow::Error>::into(DbError::NotFound),
            _ => error.into(),
        })
        .await
    }

    async fn create(tx: &mut PgTransaction<'_>, create: BlockCreate) -> Result<Block> {
        match create.kind {
            BlockKindTemplate::IngredientCollection {
                ingredient_collection,
                ..
            } => {
                let link_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "
                    INSERT INTO public.ingredient_collection_blocks (id, ingredient_collection_id)
                    VALUES ($1, $2)
                    ",
                )
                .bind(link_id)
                .bind(ingredient_collection.id)
                .execute(&mut **tx)
                .await?;

                let item_id = Uuid::new_v4();
                let item = sqlx::query(
                    "
                    INSERT INTO public.blocks (id, ingredient_collection_block_id)
                    VALUES ($1, $2)
                    RETURNING ts_created
                    ",
                )
                .bind(item_id)
                .bind(link_id)
                .fetch_one(&mut **tx)
                .await?;

                Ok(Block {
                    id: item_id,
                    ts_created: item.get("ts_created"),
                    ts_updated: None,
                    data: BlockDataTemplate {
                        kind: BlockKindTemplate::IngredientCollection {
                            link_id,
                            ingredient_collection: IngredientCollectionReference {
                                id: ingredient_collection.id,
                                ..Default::default()
                            },
                        },
                    },
                })
            }

            BlockKindTemplate::Markdown { markdown, .. } => {
                let link_id = Uuid::new_v4();
                let _ = sqlx::query(
                    "
                    INSERT INTO public.markdown_blocks (id, markdown_id)
                    VALUES ($1, $2)
                    ",
                )
                .bind(link_id)
                .bind(markdown.id)
                .execute(&mut **tx)
                .await?;

                let item_id = Uuid::new_v4();
                let item = sqlx::query(
                    "
                    INSERT INTO public.blocks (id, markdown_block_id)
                    VALUES ($1, $2)
                    RETURNING ts_created
                    ",
                )
                .bind(item_id)
                .bind(link_id)
                .fetch_one(&mut **tx)
                .await?;

                Ok(Block {
                    id: item_id,
                    ts_created: item.get("ts_created"),
                    ts_updated: None,
                    data: BlockDataTemplate {
                        kind: BlockKindTemplate::Markdown {
                            link_id,
                            markdown: MarkdownReference {
                                id: markdown.id,
                                ..Default::default()
                            },
                        },
                    },
                })
            }
        }
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        update: BlockUpdate,
    ) -> Result<Block> {
        let mut item = Self::get_by_id(&mut **tx, id).await?;

        match &mut item.data.kind {
            BlockKindTemplate::IngredientCollection {
                link_id,
                ingredient_collection: current,
            } => match update.kind {
                Some(BlockKindTemplate::IngredientCollection {
                    ingredient_collection: update,
                    ..
                }) => {
                    if let Some(update) = update {
                        current.id = update.id
                    }

                    sqlx::query(
                        "
                         UPDATE public.ingredient_collection_blocks
                         SET ingredient_collection_id = $2,
                             ts_updated = NOW()
                         WHERE id = $1
                         ",
                    )
                    .bind(link_id.clone())
                    .bind(current.id)
                    .execute(&mut **tx)
                    .await?;

                    // Data might have been invalidated, just leave it out
                    current.data = None;
                }

                // List item type cannot be changed
                Some(_) => return Err((DbError::InvalidOperation).into()),

                // Nothing to update
                _ => {}
            },

            BlockKindTemplate::Markdown {
                link_id,
                markdown: current,
            } => match update.kind {
                Some(BlockKindTemplate::Markdown {
                    markdown: update, ..
                }) => {
                    if let Some(update) = update {
                        current.id = update.id
                    }

                    sqlx::query(
                        "
                         UPDATE public.markdown_blocks
                         SET markdown_id = $2,
                             ts_updated = NOW()
                         WHERE id = $1
                         ",
                    )
                    .bind(link_id.clone())
                    .bind(current.id)
                    .execute(&mut **tx)
                    .await?;

                    // Data might have been invalidated, just leave it out
                    current.data = None;
                }

                // List item type cannot be changed
                Some(_) => return Err((DbError::InvalidOperation).into()),

                // Nothing to update
                _ => {}
            },
        };

        sqlx::query(
            "
             UPDATE public.blocks
             SET ts_updated = NOW()
             WHERE id = $1
             ",
        )
        .bind(id)
        .execute(&mut **tx)
        .await?;

        Ok(item)
    }

    async fn delete_by_id(tx: &mut PgTransaction<'_>, id: &Uuid) -> Result<()> {
        let item = Self::get_by_id(&mut **tx, id).await?;

        let delete_link_query = match item.data.kind {
            BlockKindTemplate::IngredientCollection { link_id, .. } => sqlx::query(
                "
                DELETE FROM public.ingredient_collection_blocks
                WHERE id = $1
                ",
            )
            .bind(link_id)
            .execute(&mut **tx),

            BlockKindTemplate::Markdown { link_id, .. } => sqlx::query(
                "
                DELETE FROM public.markdown_blocks
                WHERE id = $1
                ",
            )
            .bind(link_id)
            .execute(&mut **tx),
        };

        // We are relying on cascaded deletion of the main item
        if delete_link_query.await?.rows_affected() == 0 {
            return Err((DbError::InvalidContent).into());
        }

        Ok(())
    }
}
