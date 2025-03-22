import { get, host, post, type DataParams } from '..';
import type { Pagination } from '../types';

function url(searchParams: URLSearchParams = new URLSearchParams()) {
	const searchParamsFiltered = new URLSearchParams(
		searchParams
			.entries()
			.filter(([key]) => ['name'].includes(key))
			.toArray()
	);

	if (searchParamsFiltered.size > 0) {
		return `${host}/api/products?${searchParamsFiltered.toString()}`;
	} else {
		return `${host}/api/products`;
	}
}

export type GetResponse = {
	pagination: Pagination;
	data: {
		id: string;
		data: {
			name: string;
			shopping_list_item_links: {
				id: string;
			}[];
		};
	}[];
};

export type PostRequest = {
	data: {
		name: string;
	}[];
};

export type PostResponse = {
	data: {
		id: string;
	}[];
};

export default {
	url: (searchParams?: URLSearchParams) => {
		return url(searchParams);
	},

	get: (searchParams?: URLSearchParams, params?: DataParams): Promise<GetResponse> => {
		return get(url(searchParams), params);
	},

	post: (body: PostRequest, params?: DataParams): Promise<PostResponse> => {
		return post(url(), body, params);
	}
};
