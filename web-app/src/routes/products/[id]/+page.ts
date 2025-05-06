import product from '$lib/data/products/resource';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch, parent }) => {
	const product_data = await product.get(params.id, {
		fetcher: fetch
	});

	if (!product_data.ok) {
		error(product_data.response.status, product_data.response.statusText);
	}

	const parent_data = await parent();

	return {
		product: product_data.data,
		lists: parent_data.lists
	};
};
