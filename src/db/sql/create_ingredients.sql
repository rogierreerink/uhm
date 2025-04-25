DO $$ BEGIN

    -- Table: ingredients

    CREATE TABLE IF NOT EXISTS public.ingredients ();

    ALTER TABLE public.ingredients
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS ingredient_collection_id UUID NOT NULL REFERENCES public.ingredient_collections (id),
        ADD IF NOT EXISTS product_id UUID REFERENCES public.products (id);

END $$