DO
$$
    BEGIN
        /* table: ingredient_collections
         */

        IF (SELECT NOT EXISTS (SELECT
                               FROM pg_tables
                               WHERE schemaname = 'public'
                                 AND tablename = 'ingredient_collections')) THEN
            CREATE TABLE public.ingredient_collections
            (
                id uuid NOT NULL,
                ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ts_updated TIMESTAMP WITH TIME ZONE,
                PRIMARY KEY (id)
            );

            ALTER TABLE IF EXISTS public.ingredient_collections
                OWNER to postgres;

            RAISE NOTICE 'ingredient_collections: table created';
        ELSE
            RAISE NOTICE 'ingredient_collections: table already exists';
        END IF;

        INSERT INTO public.ingredient_collections (id)
        VALUES ('2e6b8e1d-4712-47d1-98d8-32299e092d0f')
        ON CONFLICT (ID) DO NOTHING;

        RAISE NOTICE 'ingredient_collections: test data upserted';

        /* table: ingredients
         */

        IF (SELECT NOT EXISTS (SELECT
                               FROM pg_tables
                               WHERE schemaname = 'public'
                                 AND tablename = 'ingredients')) THEN
            CREATE TABLE public.ingredients
            (
                id uuid NOT NULL,
                ingredient_collection_id UUID NOT NULL REFERENCES public.ingredient_collections (id),
                product_id UUID NOT NULL REFERENCES public.products (id),
                ts_created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
                ts_updated TIMESTAMP WITH TIME ZONE,
                PRIMARY KEY (id)
            );

            ALTER TABLE IF EXISTS public.ingredients
                OWNER to postgres;

            RAISE NOTICE 'ingredients: table created';
        ELSE
            RAISE NOTICE 'ingredients: table already exists';
        END IF;

        INSERT INTO public.ingredients (id, ingredient_collection_id, product_id)
        VALUES ('9ffbf5f7-aff0-4ab2-808e-27ce896db94c', '2e6b8e1d-4712-47d1-98d8-32299e092d0f', '49b6b1f9-541d-4dff-a3d5-0195cb0f6541'),
               ('914e3c07-2caa-40f8-a90a-fa91e752fbdf', '2e6b8e1d-4712-47d1-98d8-32299e092d0f', '170f015f-d9ce-4f90-b461-efcbaae6891f'),
               ('599315d2-b122-4e92-97bd-a3b3fdf9844b', '2e6b8e1d-4712-47d1-98d8-32299e092d0f', 'e929b8ee-c45f-4ebe-90dc-5366b4e5b36e')
        ON CONFLICT (ID) DO UPDATE
            SET product_id = EXCLUDED.product_id;

        RAISE NOTICE 'ingredients: test data upserted';
    END
$$