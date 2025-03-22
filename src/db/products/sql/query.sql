SELECT products.id,
       products.name,
       products.ts_created,
       products.ts_updated,
       shopping_list.id AS shopping_list_item_id
FROM public.products
         LEFT JOIN public.shopping_list_product_links shopping_list_item_links
                   ON products.id = shopping_list_item_links.product_id
         LEFT JOIN public.shopping_list shopping_list
                   ON shopping_list_item_links.id = shopping_list.product_link_id
WHERE CAST($1 AS VARCHAR) IS NULL
   OR (CAST($1 AS VARCHAR) IS NOT NULL AND name ~* $1)
ORDER BY name, id, shopping_list_item_id