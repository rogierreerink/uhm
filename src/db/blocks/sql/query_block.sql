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
ORDER BY blocks.id