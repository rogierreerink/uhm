DO $$ BEGIN

    -- Table: product_list_items

    CREATE TABLE IF NOT EXISTS public.product_list_items ();

    ALTER TABLE public.product_list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS product_id UUID REFERENCES public.products (id),
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE;

    INSERT INTO public.product_list_items (
        id, product_id
    ) VALUES
        ('be6cdaec-9ec0-4f12-9ef3-daddde4fd9d9', 'cfae1b2c-97fc-4af6-b282-6d6de5db39a3'),
        ('12d1fb25-dc10-4213-9f15-c11fbce04ab2', '49b6b1f9-541d-4dff-a3d5-0195cb0f6541'),
        ('277a2cf7-e69d-4459-a818-59955da0635b', 'e929b8ee-c45f-4ebe-90dc-5366b4e5b36e')
    ON CONFLICT (id) DO UPDATE SET
        product_id = EXCLUDED.product_id;

    -- Table: temporary_list_items

    CREATE TABLE IF NOT EXISTS public.temporary_list_items ();

    ALTER TABLE public.temporary_list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS name VARCHAR(256) NOT NULL,
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE;

    INSERT INTO public.temporary_list_items (
        id, name
    ) VALUES
        ('a987bf16-163b-4b3d-ba03-e372673aff8a', 'Bordercollies')
    ON CONFLICT (id) DO UPDATE SET
        name = EXCLUDED.name;

    -- Table: list_items

    CREATE TABLE IF NOT EXISTS public.list_items ();

    ALTER TABLE public.list_items
        ADD IF NOT EXISTS id UUID NOT NULL PRIMARY KEY,
        ADD IF NOT EXISTS checked BOOLEAN NOT NULL DEFAULT FALSE,
        ADD IF NOT EXISTS temporary_list_item_id UUID UNIQUE REFERENCES public.temporary_list_items (id),
        ADD IF NOT EXISTS product_list_item_id UUID UNIQUE REFERENCES public.product_list_items (id),
        ADD IF NOT EXISTS ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        ADD IF NOT EXISTS ts_updated TIMESTAMP WITH TIME ZONE,

        DROP CONSTRAINT IF EXISTS holds_exactly_one_list_item_reference,
        ADD CONSTRAINT holds_exactly_one_list_item_reference CHECK (
            (temporary_list_item_id IS NOT NULL)::INTEGER +
            (product_list_item_id IS NOT NULL)::INTEGER = 1
        );

    INSERT INTO public.list_items (
        id, temporary_list_item_id, product_list_item_id
    ) VALUES
        ('47e30ec2-3880-41db-9d0d-55a1b046de44', 'a987bf16-163b-4b3d-ba03-e372673aff8a', NULL),
        ('18c4c387-bd81-4cf8-bea5-667820ee0b1e', NULL, '12d1fb25-dc10-4213-9f15-c11fbce04ab2'),
        ('7360d04e-bd32-4e0f-bb5c-57d839831767', NULL, '277a2cf7-e69d-4459-a818-59955da0635b')
    ON CONFLICT (id) DO UPDATE SET
        temporary_list_item_id = EXCLUDED.temporary_list_item_id,
        product_list_item_id = EXCLUDED.product_list_item_id;

END $$