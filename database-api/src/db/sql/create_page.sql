DO $$ BEGIN

    -- Type: page types

    CREATE TYPE page_type AS ENUM (
        'recipe'
    );

    -- Table: pages

    CREATE TABLE IF NOT EXISTS public.pages ();

    ALTER TABLE public.pages
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS type page_type NOT NULL,
        ADD IF NOT EXISTS name VARCHAR(256) NOT NULL;

    -- Table: page_blocks

    CREATE TABLE IF NOT EXISTS public.page_blocks ();

    ALTER TABLE public.page_blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS page_id UUID NOT NULL REFERENCES public.pages (id)
            ON DELETE CASCADE,
        ADD IF NOT EXISTS block_id UUID REFERENCES public.blocks (id)
            ON DELETE CASCADE,
        ADD IF NOt EXISTS sequence_number INTEGER NOT NULL;

END $$