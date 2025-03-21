DO
$$
    BEGIN
        /* table: block_ingredient_list
         */

        IF (SELECT NOT EXISTS (SELECT
                               FROM pg_tables
                               WHERE schemaname = 'public'
                                 AND tablename = 'block_ingredient_list')) THEN
            CREATE TABLE public.block_ingredient_list
            (
                id uuid NOT NULL,
                PRIMARY KEY (id)
            );

            ALTER TABLE IF EXISTS public.block_ingredient_list
                OWNER to postgres;

            RAISE NOTICE 'block_ingredient_list: table created';
        ELSE
            RAISE NOTICE 'block_ingredient_list: table already exists';
        END IF;

        INSERT INTO public.block_ingredient_list (id, name)
        VALUES ('cfae1b2c-97fc-4af6-b282-6d6de5db39a3', 'Pasta'),
               ('49b6b1f9-541d-4dff-a3d5-0195cb0f6541', 'Milk'),
               ('e929b8ee-c45f-4ebe-90dc-5366b4e5b36e', 'Butter'),
               ('170f015f-d9ce-4f90-b461-efcbaae6891f', 'Maple syrup'),
               ('c209cb69-8b9e-44dc-89ff-22a52ea2be27', 'Cola')
        ON CONFLICT (ID) DO UPDATE
            SET name = EXCLUDED.name;

        RAISE NOTICE 'block_ingredient_list: test data upserted';
    END
$$