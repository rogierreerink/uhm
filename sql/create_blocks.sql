DO $$ BEGIN

    -- Table: ingredient_collection_blocks

    CREATE TABLE IF NOT EXISTS public.ingredient_collection_blocks ();

    ALTER TABLE public.ingredient_collection_blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ingredient_collection_id UUID REFERENCES public.ingredient_collections (id),
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE;

    INSERT INTO public.ingredient_collection_blocks (
        id, ingredient_collection_id
    ) VALUES
        ('70b90923-1fe9-4d4f-b0c2-ddbd9ccd9f3c', '2e6b8e1d-4712-47d1-98d8-32299e092d0f')
    ON CONFLICT (id) DO UPDATE SET
        ingredient_collection_id = EXCLUDED.ingredient_collection_id;

    -- Table: paragraph_blocks

    CREATE TABLE IF NOT EXISTS public.paragraph_blocks ();

    ALTER TABLE public.paragraph_blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS text TEXT,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE;

    INSERT INTO public.paragraph_blocks (
        id, text
    ) VALUES
        ('ab5d8c03-64f9-440d-bfba-da9b564516d6', 'Hello, world!')
    ON CONFLICT (id) DO UPDATE SET
        text = EXCLUDED.text;

    -- Table: blocks

    CREATE TABLE IF NOT EXISTS public.blocks ();

    ALTER TABLE public.blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ingredient_collection_block_id UUID REFERENCES public.ingredient_collection_blocks (id),
        ADD IF NOT EXISTS paragraph_block_id UUID REFERENCES public.paragraph_blocks (id),
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        
        DROP CONSTRAINT IF EXISTS holds_exactly_one_block_reference,
        ADD CONSTRAINT holds_exactly_one_block_reference CHECK (
            (ingredient_collection_block_id IS NOT NULL)::INTEGER +
            (paragraph_block_id IS NOT NULL)::INTEGER = 1
        );

    INSERT INTO public.blocks (
        id, ingredient_collection_block_id, paragraph_block_id
    ) VALUES
        ('88741272-0aed-4157-8da8-871fc8f4fae2', NULL, 'ab5d8c03-64f9-440d-bfba-da9b564516d6'),
        ('444ede88-b2e6-4ac7-a0e2-6a50c30f4efc', '70b90923-1fe9-4d4f-b0c2-ddbd9ccd9f3c', NULL)
    ON CONFLICT (id) DO UPDATE SET
        ingredient_collection_block_id = EXCLUDED.ingredient_collection_block_id,
        paragraph_block_id = EXCLUDED.paragraph_block_id;

END $$