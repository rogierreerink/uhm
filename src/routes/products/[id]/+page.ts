import { product } from '$lib/data/products';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	return product.get(params.id, {
		fetcher: fetch
	});
};
