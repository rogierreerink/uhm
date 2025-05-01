import lists from '$lib/data/lists/collection';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
	const response = await lists.get({
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status);
	}

	return { lists: response.data };
};
