import shoppingList from '$lib/data/shopping-list/collection';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const response = await shoppingList.get({
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status);
	}

	return response.data;
};
