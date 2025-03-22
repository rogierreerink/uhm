import products from '$lib/data/products/collection';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, fetch }) => {
	const response = await products.get(url.searchParams, {
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status);
	}

	return response.data;
};
