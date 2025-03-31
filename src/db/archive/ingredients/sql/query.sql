SELECT
    ingredients.id,
    ingredients.ingredient_collection_id,
    ingredients.product_id,
    ingredients.ts_created,
    ingredients.ts_updated,
    products.name AS product_name

FROM public.ingredients
    LEFT JOIN public.products
        ON ingredients.product_id = products.id

WHERE ingredients.ingredient_collection_id = $1
ORDER BY products.name, ingredients.id