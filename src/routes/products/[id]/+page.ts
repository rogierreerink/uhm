import product from '$lib/data/products/resource';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const response = await product.get(params.id, {
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status, response.response.statusText);
	}

	return response.data;
};
