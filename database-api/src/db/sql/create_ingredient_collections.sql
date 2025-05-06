DO $$ BEGIN

    -- Table: ingredient_collections

    CREATE TABLE IF NOT EXISTS public.ingredient_collections ();

    ALTER TABLE public.ingredient_collections
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE;

END $$