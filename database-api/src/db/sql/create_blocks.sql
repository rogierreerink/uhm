DO $$ BEGIN

    -- Table: ingredient_collection_blocks

    CREATE TABLE IF NOT EXISTS public.ingredient_collection_blocks ();

    ALTER TABLE public.ingredient_collection_blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS ingredient_collection_id UUID REFERENCES public.ingredient_collections (id)
            ON DELETE CASCADE;

    -- Table: markdown_blocks

    CREATE TABLE IF NOT EXISTS public.markdown_blocks ();

    ALTER TABLE public.markdown_blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS markdown_id UUID REFERENCES public.markdown (id)
            ON DELETE CASCADE;

    -- Table: blocks

    CREATE TABLE IF NOT EXISTS public.blocks ();

    ALTER TABLE public.blocks
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,
        ADD IF NOT EXISTS ingredient_collection_block_id UUID REFERENCES public.ingredient_collection_blocks (id)
            ON DELETE CASCADE,
        ADD IF NOT EXISTS markdown_block_id UUID REFERENCES public.markdown_blocks (id)
            ON DELETE CASCADE,
        
        DROP CONSTRAINT IF EXISTS holds_exactly_one_block_reference,
        ADD CONSTRAINT holds_exactly_one_block_reference CHECK (
            (ingredient_collection_block_id IS NOT NULL)::INTEGER +
            (markdown_block_id IS NOT NULL)::INTEGER = 1
        );

END $$

-- Function/trigger: delete block references when deleting block

CREATE OR REPLACE FUNCTION public.delete_block_references()
RETURNS TRIGGER
LANGUAGE plpgsql AS $$
    DECLARE
    BEGIN
        -- Delete ingredient collection block
        IF OLD.ingredient_collection_block_id IS NOT NULL THEN
            DELETE FROM public.ingredient_collection_blocks
            WHERE id = OLD.ingredient_collection_block_id;
            RETURN OLD;
        END IF;
        
        -- Delete markdown block
        IF OLD.markdown_block_id IS NOT NULL THEN
            DELETE FROM public.markdown_blocks
            WHERE id = OLD.markdown_block_id;
            RETURN OLD;
        END IF;
    END;
$$;

CREATE OR REPLACE TRIGGER delete_block_references
AFTER DELETE ON public.blocks
FOR EACH ROW
EXECUTE FUNCTION delete_block_references();