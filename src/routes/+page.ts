import { shoppingListItems } from '$lib/data/shopping-list';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ url, fetch }) => {
	return shoppingListItems.get(url.searchParams, {
		fetcher: fetch
	});
};
