DO $$ BEGIN

    -- Table: product_list_items

    CREATE TABLE IF NOT EXISTS public.product_list_items ();

    ALTER TABLE public.product_list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS product_id UUID REFERENCES public.products (id)
            ON DELETE CASCADE;

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
        ADD IF NOT EXISTS list_id UUID NOT NULL REFERENCES public.lists (id)
            ON DELETE CASCADE,
        ADD IF NOT EXISTS product_list_item_id UUID UNIQUE REFERENCES public.product_list_items (id)
            ON DELETE CASCADE,
        ADD IF NOT EXISTS temporary_list_item_id UUID UNIQUE REFERENCES public.temporary_list_items (id)
            ON DELETE CASCADE,

        DROP CONSTRAINT IF EXISTS holds_exactly_one_list_item_reference,
        ADD CONSTRAINT holds_exactly_one_list_item_reference CHECK (
            (product_list_item_id IS NOT NULL)::INTEGER +
            (temporary_list_item_id IS NOT NULL)::INTEGER = 1
        );

END $$

-- Function/trigger: delete list item references when deleting list item

CREATE OR REPLACE FUNCTION public.delete_list_item_references()
RETURNS TRIGGER
LANGUAGE plpgsql AS $$
    DECLARE
    BEGIN
        -- Delete product list item
        IF OLD.product_list_item_id IS NOT NULL THEN
            DELETE FROM public.product_list_items
            WHERE id = OLD.product_list_item_id;
            RETURN OLD;
        END IF;
        
        -- Delete temporary list item
        IF OLD.temporary_list_item_id IS NOT NULL THEN
            DELETE FROM public.temporary_list_items
            WHERE id = OLD.temporary_list_item_id;
            RETURN OLD;
        END IF;
    END;
$$;

CREATE OR REPLACE TRIGGER delete_list_item_references
AFTER DELETE ON public.list_items
FOR EACH ROW
EXECUTE FUNCTION delete_list_item_references();