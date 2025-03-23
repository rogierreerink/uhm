import { get, host, post, type DataParams, type DataResponse } from '..';
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
		created: Date;
		updated?: Date;
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

	get: async (
		searchParams?: URLSearchParams,
		params?: DataParams
	): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(searchParams), params);

		if (!response.ok) {
			return response;
		}

		return {
			...response,
			data: {
				...response.data,
				data: response.data.data.map((item) => ({
					...item,
					created: new Date(item.created),
					updated: item.updated && new Date(item.updated)
				}))
			}
		};
	},

	post: (body: PostRequest, params?: DataParams): Promise<DataResponse<PostResponse>> => {
		return post(url(), body, params);
	}
};
