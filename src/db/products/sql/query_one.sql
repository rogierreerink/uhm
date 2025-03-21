SELECT products.id,
       products.name,
       products.ts_created,
       products.ts_updated,
       shopping_list_item_links.id AS shopping_list_item_id
FROM public.products
         LEFT JOIN public.shopping_list_product_links shopping_list_item_links
                   ON products.id = shopping_list_item_links.product_id
WHERE products.id = $1
ORDER BY shopping_list_item_id