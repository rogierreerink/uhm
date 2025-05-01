import list from '$lib/data/lists/resource';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch }) => {
	const response = await list.get(params.id, {
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status);
	}

	return response.data;
};
