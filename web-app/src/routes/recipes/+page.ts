import pages from '$lib/data/pages/collection';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, parent }) => {
	const response = await pages.get(new URLSearchParams({ type: 'recipe' }), {
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status);
	}

	return {
		recipes: response.data,
		...(await parent())
	};
};
