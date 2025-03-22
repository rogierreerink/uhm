import products from '$lib/data/products/collection';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, fetch }) => {
	return products.get(url.searchParams, {
		fetcher: fetch
	});
};
