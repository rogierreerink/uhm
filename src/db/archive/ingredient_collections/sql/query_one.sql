SELECT
    ingredient_collections.id,
    ingredient_collections.ts_created,
    ingredient_collections.ts_updated,
    ingredients.id AS ingredient_id,
    products.id AS product_id,
    products.name AS product_name

FROM public.ingredient_collections
    LEFT JOIN public.ingredients
        ON ingredient_collections.id = ingredients.ingredient_collection_id
    LEFT JOIN public.products
        ON ingredients.product_id = products.id

WHERE ingredient_collections.id = $1
ORDER BY
    ingredient_collections.id,
    products.name,
    ingredients.id