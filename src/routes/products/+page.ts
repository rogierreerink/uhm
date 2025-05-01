import products from '$lib/data/products/collection';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, fetch, parent }) => {
	const product_data = await products.get(url.searchParams, {
		fetcher: fetch
	});

	if (!product_data.ok) {
		error(product_data.response.status);
	}

	const parent_data = await parent();

	return {
		products: product_data.data,
		lists: parent_data.lists
	};
};
