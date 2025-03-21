DO
$$
    BEGIN
        /* table: shopping_list_temp_items
         */

        IF (SELECT NOT EXISTS (SELECT
                               FROM pg_tables
                               WHERE schemaname = 'public'
                                 AND tablename = 'shopping_list_temp_items')) THEN
            CREATE TABLE public.shopping_list_temp_items
            (
                id         UUID NOT NULL PRIMARY KEY,
                name       VARCHAR(256) NOT NULL,
                ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ts_updated TIMESTAMP WITH TIME ZONE
            );

            ALTER TABLE IF EXISTS public.shopping_list_temp_items
                OWNER TO postgres;

            RAISE NOTICE 'shopping_list_temp_items: table created';
        ELSE
            RAISE NOTICE 'shopping_list_temp_items: table already exists';
        END IF;

        INSERT INTO public.shopping_list_temp_items (id, name)
        VALUES ('a987bf16-163b-4b3d-ba03-e372673aff8a', 'Bordercollies')
        ON CONFLICT (ID) DO UPDATE
            SET name = EXCLUDED.name;

        RAISE NOTICE 'shopping_list_temp_items: test data upserted';

        /* table: shopping_list_product_links
         */

        IF (SELECT NOT EXISTS (SELECT
                               FROM pg_tables
                               WHERE schemaname = 'public'
                                 AND tablename = 'shopping_list_product_links')) THEN
            CREATE TABLE public.shopping_list_product_links
            (
                id         UUID NOT NULL PRIMARY KEY,
                product_id UUID REFERENCES public.products (id),
                ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ts_updated TIMESTAMP WITH TIME ZONE
            );

            ALTER TABLE IF EXISTS public.shopping_list_product_links
                OWNER TO postgres;

            RAISE NOTICE 'shopping_list_product_links: table created';
        ELSE
            RAISE NOTICE 'shopping_list_product_links: table already exists';
        END IF;

        INSERT INTO public.shopping_list_product_links (id, product_id)
        VALUES ('be6cdaec-9ec0-4f12-9ef3-daddde4fd9d9', 'cfae1b2c-97fc-4af6-b282-6d6de5db39a3'),
               ('12d1fb25-dc10-4213-9f15-c11fbce04ab2', '49b6b1f9-541d-4dff-a3d5-0195cb0f6541'),
               ('277a2cf7-e69d-4459-a818-59955da0635b', 'e929b8ee-c45f-4ebe-90dc-5366b4e5b36e')
        ON CONFLICT (ID) DO UPDATE
            SET product_id = EXCLUDED.product_id;

        RAISE NOTICE 'shopping_list_product_links: test data upserted';

        /* table: shopping_list
         */

        IF (SELECT NOT EXISTS (SELECT
                               FROM pg_tables
                               WHERE schemaname = 'public'
                                 AND tablename = 'shopping_list')) THEN
            CREATE TABLE public.shopping_list
            (
                id              UUID    NOT NULL PRIMARY KEY,
                in_cart         BOOLEAN NOT NULL DEFAULT FALSE,
                temp_item_id    UUID UNIQUE REFERENCES public.shopping_list_temp_items (id),
                product_link_id UUID UNIQUE REFERENCES public.shopping_list_product_links (id),
                ts_created      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ts_updated      TIMESTAMP WITH TIME ZONE,
                CHECK (
                    (temp_item_id IS NOT NULL)::INTEGER +
                    (product_link_id IS NOT NULL)::INTEGER = 1
                )
            );

            ALTER TABLE IF EXISTS public.shopping_list
                OWNER TO postgres;

            RAISE NOTICE 'shopping_list: table created';
        ELSE
            RAISE NOTICE 'shopping_list: table already exists';
        END IF;

        INSERT INTO public.shopping_list (id, temp_item_id, product_link_id)
        VALUES ('47e30ec2-3880-41db-9d0d-55a1b046de44', 'a987bf16-163b-4b3d-ba03-e372673aff8a', NULL),
               ('18c4c387-bd81-4cf8-bea5-667820ee0b1e', NULL, '12d1fb25-dc10-4213-9f15-c11fbce04ab2'),
               ('7360d04e-bd32-4e0f-bb5c-57d839831767', NULL, '277a2cf7-e69d-4459-a818-59955da0635b')
        ON CONFLICT (ID) DO UPDATE
            SET temp_item_id    = EXCLUDED.temp_item_id,
                product_link_id = EXCLUDED.product_link_id;

        RAISE NOTICE 'shopping_list: test data upserted';
    END
$$