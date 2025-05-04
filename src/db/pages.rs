use std::pin::Pin;

use anyhow::Result;
use chrono::{DateTime, Utc};
use futures_util::{stream::Peekable, Stream, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::FromRow, PgExecutor, PgPool, PgTransaction, Row};
use uuid::Uuid;

use crate::{
    db::blocks::BlockTemplate,
    utilities::{
        markdown::markdown_to_html,
        modifier::{Create, Modifier, Query, Reference, Update},
    },
};

use super::{
    blocks::{BlockDataTemplate, BlockKindTemplate, BlockReference},
    ingredient_collections::{IngredientCollectionDataTemplate, IngredientCollectionReference},
    ingredients::{IngredientDataTemplate, IngredientReference},
    markdown::{MarkdownDataTemplate, MarkdownReference},
    products::{ProductDataTemplate, ProductReference},
    DbError,
};

#[trait_variant::make(Send)]
pub trait PageDb {
    async fn get_multiple(&mut self, params: SearchParams) -> Result<Vec<Page>>;
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
    pub r#type: M::Data<PageType>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub name: M::Data<String>,
    #[serde(skip_serializing_if = "M::skip_data")]
    pub blocks: M::Data<Vec<PageBlockTemplate<M>>>,
}

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize)]
#[sqlx(type_name = "page_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    Recipe,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PageBlockTemplate<M: Modifier> {
    #[serde(skip)]
    pub link_id: M::Data<Uuid>,
    #[serde(flatten)]
    pub block: M::Data<BlockReference>,
}

#[derive(Default, Debug, Deserialize)]
pub struct SearchParams {
    pub r#type: Option<PageType>,
}

impl FromRow<'_, PgRow> for Page {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, sqlx::Error> {
        Ok(Self {
            id: row.get("id"),
            ts_created: row.get("ts_created"),
            ts_updated: row.get("ts_updated"),
            data: PageDataTemplate {
                r#type: row.get("type"),
                name: row.get("name"),
                blocks: Vec::new(),
            },
        })
    }
}

macro_rules! next_matches_first {
    ($stream:ident, $first:ident, $($column_name:expr),+) => {
        if let Some(Ok(next)) = $stream.as_mut().peek().await {
            $(Some(next.get::<Uuid, _>($column_name)) == $first.get($column_name)) && +
        } else {
            false
        }
    };
}

impl Page {
    async fn collect_pages(
        stream: impl Stream<Item = Result<PgRow, sqlx::Error>>,
        summary: bool,
    ) -> Result<Vec<Page>> {
        let mut stream = std::pin::pin!(stream.peekable());
        let mut items = Vec::new();
        loop {
            let next = match stream.as_mut().try_next().await? {
                Some(next) => next,
                None => return Ok(items),
            };

            items.push(Self::collect_page(&next, &mut stream, summary).await?);
        }
    }

    async fn collect_page(
        first: &PgRow,
        rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
        summary: bool,
    ) -> Result<Page> {
        Ok(Page {
            id: first.get("id"),
            ts_created: first.get("ts_created"),
            ts_updated: first.get("ts_updated"),
            data: PageDataTemplate {
                r#type: first.get("type"),
                name: first.get("name"),
                blocks: if !summary {
                    let mut items = vec![Self::collect_page_block(first, rest).await?];
                    loop {
                        if !next_matches_first!(rest, first, "id") {
                            break items;
                        }

                        let next = match rest.try_next().await? {
                            Some(next) => next,
                            None => break items,
                        };

                        items.push(Self::collect_page_block(&next, rest).await?);
                    }
                } else {
                    Vec::new()
                },
            },
        })
    }

    async fn collect_page_block(
        first: &PgRow,
        rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<PageBlockTemplate<Query>> {
        Ok(PageBlockTemplate {
            link_id: first.get("page_block_id"),
            block: BlockReference {
                id: first.get("block_id"),
                data: Some(BlockDataTemplate::<Reference> {
                    kind: Some({
                        if let Some(id) = first.get("ingredient_collection_block_id") {
                            BlockKindTemplate::IngredientCollection {
                                link_id: id,
                                ingredient_collection: Some(
                                    Self::collect_ingredient_collection(first, rest).await?,
                                ),
                            }
                        } else if let Some(id) = first.get("markdown_block_id") {
                            BlockKindTemplate::Markdown {
                                link_id: id,
                                markdown: Some(Self::collect_markdown(first, rest).await?),
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

    async fn collect_ingredient_collection(
        first: &PgRow,
        rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<IngredientCollectionReference> {
        Ok(IngredientCollectionReference {
            id: first.get("ingredient_collection_id"),
            data: Some(IngredientCollectionDataTemplate {
                ingredients: Some({
                    let mut items = vec![Self::collect_ingredient(first, rest).await?];
                    loop {
                        if !next_matches_first!(rest, first, "id", "ingredient_collection_id") {
                            break items;
                        }

                        let next = match rest.try_next().await? {
                            Some(next) => next,
                            None => break items,
                        };

                        items.push(Self::collect_ingredient(&next, rest).await?);
                    }
                }),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    async fn collect_ingredient(
        first: &PgRow,
        _rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<IngredientReference> {
        Ok(IngredientReference {
            id: first.get("ingredient_id"),
            data: Some(IngredientDataTemplate {
                product: Some(ProductReference {
                    id: first.get("product_id"),
                    data: Some(ProductDataTemplate {
                        name: Some(first.get("product_name")),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            }),
            ..Default::default()
        })
    }

    async fn collect_markdown(
        first: &PgRow,
        _rest: &mut Pin<&mut Peekable<impl Stream<Item = Result<PgRow, sqlx::Error>>>>,
    ) -> Result<MarkdownReference> {
        Ok(MarkdownReference {
            id: first.get("markdown_id"),
            data: Some({
                let markdown = first.get::<String, _>("markdown");
                let html = markdown_to_html(&markdown);
                MarkdownDataTemplate {
                    markdown: Some(markdown),
                    html: Some(html),
                    ..Default::default()
                }
            }),
            ..Default::default()
        })
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
    async fn get_multiple(&mut self, params: SearchParams) -> Result<Vec<Page>> {
        let mut conn = self.pool.acquire().await?;
        let stream = sqlx::query(
            "
            SELECT
                pages.id,
                pages.ts_created,
                pages.ts_updated,
                pages.type,
                pages.name
            FROM public.pages
            WHERE pages.type = $1 OR $1 IS NULL
            ORDER BY pages.name
            ",
        )
        .bind(params.r#type)
        .fetch(&mut *conn);

        Page::collect_pages(stream, true).await
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
        let stream = sqlx::query(
            "
            SELECT
                pages.id,
                pages.ts_created,
                pages.ts_updated,
                pages.type,
                pages.name,
                page_blocks.id AS page_block_id,
                blocks.id AS block_id,
                ingredient_collection_blocks.id AS ingredient_collection_block_id,
                ingredient_collections.id AS ingredient_collection_id,
                ingredients.id AS ingredient_id,
                products.id AS product_id,
                products.name AS product_name,
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
                LEFT JOIN public.ingredients
                    ON ingredient_collections.id = ingredients.ingredient_collection_id
                LEFT JOIN public.products
                    ON ingredients.product_id = products.id

                LEFT JOIN public.markdown_blocks
                    ON blocks.markdown_block_id = markdown_blocks.id
                LEFT JOIN public.markdown
                    ON markdown_blocks.markdown_id = markdown.id

            WHERE pages.id = $1
            ORDER BY
                page_blocks.sequence_number,
                products.name
            ",
        )
        .bind(id)
        .fetch(executor);

        match Page::collect_pages(stream, false).await?.pop() {
            Some(item) => Ok(item),
            None => Err((DbError::NotFound).into()),
        }
    }

    async fn create(tx: &mut PgTransaction<'_>, create: PageCreate) -> Result<Page> {
        let item_id = Uuid::new_v4();
        let mut item: Page = sqlx::query_as(
            "
            INSERT INTO public.pages (id, type, name)
            VALUES ($1, $2, $3)
            RETURNING id, ts_created, ts_updated, type, name
            ",
        )
        .bind(item_id)
        .bind(create.r#type)
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

        if let Some(ty) = update.r#type {
            item.data.r#type = ty;
        }
        if let Some(name) = update.name {
            item.data.name = name;
        }

        let row = sqlx::query(
            "
            UPDATE public.pages
            SET type = $2,
                name = $3,
                ts_updated = NOW()
            WHERE id = $1
            RETURNING ts_updated
            ",
        )
        .bind(id)
        .bind(item.data.r#type.clone())
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
