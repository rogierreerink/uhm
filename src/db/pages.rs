use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{Stream, TryStreamExt};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::{
    db::blocks::BlockTemplate,
    utilities::modifier::{Create, Modifier, Query, Reference, Update},
};

use super::{
    blocks::{BlockDataTemplate, BlockKindTemplate, BlockReference},
    ingredient_collections::{IngredientCollectionDataTemplate, IngredientCollectionReference},
    markdown::{MarkdownDataTemplate, MarkdownReference},
    DbError,
};

#[trait_variant::make(Send)]
pub trait PageDb {
    async fn get_multiple(&mut self) -> Result<Vec<Page>>;
    async fn get_by_id(&mut self, id: &Uuid) -> Result<Page>;
    async fn create_multiple(&mut self, items: Vec<PageCreate>) -> Result<Vec<Page>>;
    async fn update_by_id(&mut self, id: &Uuid, item: PageUpdate) -> Result<Page>;
    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()>;
}

pub type Page = PageTemplate<Query>;
pub type PageCreate = PageDataTemplate<Create>;
pub type PageUpdate = PageDataTemplate<Update>;
pub type PageReference = PageTemplate<Reference>;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PageTemplate<M: Modifier> {
    pub id: M::Key<Uuid>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_created: M::Meta<DateTime<Utc>>,
    #[serde(skip_serializing_if = "M::skip_meta")]
    pub ts_updated: M::Meta<Option<DateTime<Utc>>>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub data: M::Data<PageDataTemplate<M>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PageDataTemplate<M: Modifier> {
    #[serde(skip_serializing_if = "M::skip_data")]
    pub name: M::Data<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "M::skip_data")]
    pub blocks: M::Data<Vec<PageBlockTemplate<M>>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PageBlockTemplate<M: Modifier> {
    #[serde(skip)]
    pub link_id: M::Data<Uuid>,
    #[serde(flatten)]
    pub block: M::Data<BlockReference>,
}

impl FromRow<'_, PgRow> for Page {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: PageDataTemplate {
                name: row.get("name"),
                blocks: Vec::new(),
            },
        })
    }
}

impl FromRow<'_, PgRow> for PageBlockTemplate<Query> {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            link_id: row.get("page_block_id"),
            block: BlockReference {
                id: row.get("block_id"),
                data: Some(BlockDataTemplate::<Reference> {
                    kind: Some({
                        if let Some(id) = row.get("ingredient_collection_block_id") {
                            BlockKindTemplate::IngredientCollection {
                                link_id: id,
                                ingredient_collection: Some(IngredientCollectionReference {
                                    id: row.get("ingredient_collection_id"),
                                    data: Some(IngredientCollectionDataTemplate {
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }),
                            }
                        } else if let Some(id) = row.get("markdown_block_id") {
                            BlockKindTemplate::Markdown {
                                link_id: id,
                                markdown: Some(MarkdownReference {
                                    id: row.get("markdown_id"),
                                    data: Some(MarkdownDataTemplate {
                                        markdown: row.get("markdown"),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                }),
                            }
                        } else {
                            panic!("unreachable!")
                        }
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        })
    }
}

impl Page {
    async fn try_item_from_stream(
        rows: &mut (impl Stream<Item = Result<PgRow, sqlx::Error>> + Unpin),
    ) -> Result<Option<Self>> {
        let mut item = Option::None;

        while let Some(row) = rows.try_next().await? {
            let item = item.get_or_insert(Page::from_row(&row)?);

            if let Some(_) = row.get::<Option<Uuid>, _>("block_id") {
                item.data.blocks.push(PageBlockTemplate::from_row(&row)?);
            }
        }

        Ok(item)
    }

    async fn try_items_from_stream(
        rows: &mut (impl Stream<Item = Result<PgRow, sqlx::Error>> + Unpin),
    ) -> Result<Vec<Self>> {
        let mut items = IndexMap::<Uuid, _>::new();

        while let Some(row) = rows.try_next().await? {
            let item = items.entry(row.get("id")).or_insert(Page::from_row(&row)?);

            if let Some(_) = row.get::<Option<Uuid>, _>("block_id") {
                item.data.blocks.push(PageBlockTemplate::from_row(&row)?);
            }
        }

        Ok(items.into_values().collect())
    }
}

pub struct PageDbPostgres<'a> {
    pool: &'a PgPool,
}

impl<'a> PageDbPostgres<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl PageDb for PageDbPostgres<'_> {
    async fn get_multiple(&mut self) -> Result<Vec<Page>> {
        let mut conn = self.pool.acquire().await?;
        let mut stream = sqlx::query(
            "
            SELECT
                pages.id,
                pages.ts_created,
                pages.ts_updated,
                pages.name,
                page_blocks.id AS page_block_id,
                blocks.id AS block_id,
                ingredient_collection_blocks.id AS ingredient_collection_block_id,
                ingredient_collections.id AS ingredient_collection_id,
                markdown_blocks.id AS markdown_block_id,
                markdown.id AS markdown_id,
                markdown.markdown AS markdown

            FROM public.pages
                LEFT JOIN public.page_blocks
                    ON pages.id = page_blocks.page_id
                LEFT JOIN public.blocks
                    ON page_blocks.block_id = blocks.id

                LEFT JOIN public.ingredient_collection_blocks
                    ON blocks.ingredient_collection_block_id = ingredient_collection_blocks.id
                LEFT JOIN public.ingredient_collections
                    ON ingredient_collection_blocks.ingredient_collection_id = ingredient_collections.id

                LEFT JOIN public.markdown_blocks
                    ON blocks.markdown_block_id = markdown_blocks.id
                LEFT JOIN public.markdown
                    ON markdown_blocks.markdown_id = markdown.id

            ORDER BY
                COALESCE(pages.ts_updated, pages.ts_created) DESC,
                page_blocks.sequence_number
            ",
        )
        .fetch(&mut *conn);

        Page::try_items_from_stream(&mut stream).await
    }

    async fn get_by_id(&mut self, id: &Uuid) -> Result<Page> {
        let mut conn = self.pool.acquire().await?;

        Self::get_by_id(&mut *conn, id).await
    }

    async fn create_multiple(&mut self, items: Vec<PageCreate>) -> Result<Vec<Page>> {
        let mut tx = self.pool.begin().await?;
        let mut created = Vec::new();

        for item in items {
            match Self::create(&mut tx, item).await {
                Ok(item) => created.push(item),
                Err(error) => {
                    tx.commit().await?;
                    return Err(error.into());
                }
            };
        }

        tx.commit().await?;

        Ok(created)
    }

    async fn update_by_id(&mut self, id: &Uuid, item: PageUpdate) -> Result<Page> {
        let mut tx = self.pool.begin().await?;

        let updated = match Self::update_by_id(&mut tx, id, item).await {
            Ok(item) => item,
            Err(error) => {
                tx.commit().await?;
                return Err(error.into());
            }
        };

        tx.commit().await?;

        Ok(updated)
    }

    async fn delete_by_id(&mut self, id: &Uuid) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        // Relying on cascaded delete regarding corresponding page blocks
        if sqlx::query(
            "
            DELETE FROM public.pages
            WHERE id = $1
            ",
        )
        .bind(id)
        .execute(&mut *conn)
        .await?
        .rows_affected()
            == 0
        {
            return Err((DbError::NotFound).into());
        }

        Ok(())
    }
}

impl PageDbPostgres<'_> {
    async fn get_by_id<'c, E>(executor: E, id: &Uuid) -> Result<Page>
    where
        E: PgExecutor<'c>,
    {
        let mut stream = sqlx::query(
            "
            SELECT
                pages.id,
                pages.ts_created,
                pages.ts_updated,
                pages.name,
                page_blocks.id AS page_block_id,
                blocks.id AS block_id,
                ingredient_collection_blocks.id AS ingredient_collection_block_id,
                ingredient_collections.id AS ingredient_collection_id,
                markdown_blocks.id AS markdown_block_id,
                markdown.id AS markdown_id,
                markdown.markdown AS markdown

            FROM public.pages
                LEFT JOIN public.page_blocks
                    ON pages.id = page_blocks.page_id
                LEFT JOIN public.blocks
                    ON page_blocks.block_id = blocks.id

                LEFT JOIN public.ingredient_collection_blocks
                    ON blocks.ingredient_collection_block_id = ingredient_collection_blocks.id
                LEFT JOIN public.ingredient_collections
                    ON ingredient_collection_blocks.ingredient_collection_id = ingredient_collections.id

                LEFT JOIN public.markdown_blocks
                    ON blocks.markdown_block_id = markdown_blocks.id
                LEFT JOIN public.markdown
                    ON markdown_blocks.markdown_id = markdown.id
                    
            WHERE pages.id = $1
            ORDER BY page_blocks.sequence_number
            ",
        )
        .bind(id)
        .fetch(executor);

        match Page::try_item_from_stream(&mut stream).await? {
            Some(item) => Ok(item),
            None => Err((DbError::NotFound).into()),
        }
    }

    async fn create(tx: &mut PgTransaction<'_>, create: PageCreate) -> Result<Page> {
        let item_id = Uuid::new_v4();
        let mut item: Page = sqlx::query_as(
            "
            INSERT INTO public.pages (id, name)
            VALUES ($1, $2)
            RETURNING id, ts_created, ts_updated, name
            ",
        )
        .bind(item_id)
        .bind(create.name)
        .fetch_one(&mut **tx)
        .await?;

        let mut seq = 0;
        for page_block in create.blocks {
            let link_id = Uuid::new_v4();
            let _ = sqlx::query(
                "
                INSERT INTO public.page_blocks (id, page_id, block_id, sequence_number)
                VALUES ($1, $2, $3, $4)
                ",
            )
            .bind(link_id)
            .bind(item_id)
            .bind(page_block.block.id)
            .bind(seq)
            .execute(&mut **tx)
            .await?;

            item.data.blocks.push(PageBlockTemplate {
                link_id,
                block: BlockTemplate {
                    id: page_block.block.id,
                    ..Default::default()
                },
            });

            seq += 1;
        }

        Ok(item)
    }

    async fn update_by_id(
        tx: &mut PgTransaction<'_>,
        id: &Uuid,
        update: PageUpdate,
    ) -> Result<Page> {
        let mut item = Self::get_by_id(&mut **tx, id).await?;

        if let Some(name) = update.name {
            item.data.name = name;
        }

        let row = sqlx::query(
            "
            UPDATE public.pages
            SET name = $2,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING ts_updated
            ",
        )
        .bind(id)
        .bind(item.data.name.clone())
        .fetch_one(&mut **tx)
        .await?;

        item.ts_updated = row.get("ts_updated");

        let mut updated_blocks = Vec::new();

        if let Some(blocks) = update.blocks {
            for page_block in &item.data.blocks {
                sqlx::query(
                    "
                    DELETE FROM public.page_blocks
                    WHERE id = $1
                    ",
                )
                .bind(page_block.link_id)
                .execute(&mut **tx)
                .await?;
            }

            let mut seq = 0;
            for page_block in blocks {
                if let Some(block) = page_block.block {
                    let link_id = Uuid::new_v4();
                    let _ = sqlx::query(
                        "
                        INSERT INTO public.page_blocks (id, page_id, block_id, sequence_number)
                        VALUES ($1, $2, $3, $4)
                        ",
                    )
                    .bind(link_id)
                    .bind(id)
                    .bind(block.id)
                    .bind(seq)
                    .execute(&mut **tx)
                    .await?;

                    updated_blocks.push(PageBlockTemplate { link_id, block });

                    seq += 1;
                }
            }

            item.data.blocks = updated_blocks;
        }

        Ok(item)
    }
}
