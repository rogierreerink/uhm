import page from '$lib/data/pages/resource';
import { error } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params, fetch, parent }) => {
	const response = await page.get(params.id, {
		fetcher: fetch
	});

	if (!response.ok) {
		error(response.response.status);
	}

	return {
		recipe: response.data,
		...(await parent())
	};
};
