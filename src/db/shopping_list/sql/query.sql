SELECT shopping_list.id,
       shopping_list.in_cart,
       shopping_list.ts_created,
       shopping_list.ts_updated,
       temp_items.id            AS temp_item_id,
       temp_items.name          AS temp_item_name,
       product_links.id         AS product_link_id,
       product_links.product_id AS product_id,
       products.name            AS product_name

FROM public.shopping_list
         LEFT JOIN public.shopping_list_temp_items temp_items
                   ON shopping_list.temp_item_id = temp_items.id
         LEFT JOIN public.shopping_list_product_links product_links
                   ON shopping_list.product_link_id = product_links.id
         LEFT JOIN public.products
                   ON product_links.product_id = products.id

ORDER BY CASE
             WHEN temp_item_id IS NOT NULL THEN temp_items.name
             WHEN product_link_id IS NOT NULL THEN products.name
             END,
         shopping_list.id