DO $$ BEGIN

    -- Table: product_list_items

    CREATE TABLE IF NOT EXISTS public.product_list_items ();

    ALTER TABLE public.product_list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS product_id UUID REFERENCES public.products (id);

    -- Table: temporary_list_items

    CREATE TABLE IF NOT EXISTS public.temporary_list_items ();

    ALTER TABLE public.temporary_list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS name VARCHAR(256) NOT NULL;

    -- Table: list_items

    CREATE TABLE IF NOT EXISTS public.list_items ();

    ALTER TABLE public.list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS checked BOOLEAN NOT NULL DEFAULT FALSE,
        ADD IF NOT EXISTS product_list_item_id UUID UNIQUE REFERENCES public.product_list_items (id),
        ADD IF NOT EXISTS temporary_list_item_id UUID UNIQUE REFERENCES public.temporary_list_items (id),

        DROP CONSTRAINT IF EXISTS holds_exactly_one_list_item_reference,
        ADD CONSTRAINT holds_exactly_one_list_item_reference CHECK (
            (product_list_item_id IS NOT NULL)::INTEGER +
            (temporary_list_item_id IS NOT NULL)::INTEGER = 1
        );

END $$