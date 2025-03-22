import shoppingList from '$lib/data/shopping-list/collection';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	return shoppingList.get({
		fetcher: fetch
	});
};
