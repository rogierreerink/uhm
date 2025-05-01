import { get, host, post, type DataParams, type DataResponse } from '..';

function url(searchParams: URLSearchParams = new URLSearchParams()) {
	searchParams = new URLSearchParams(
		searchParams
			.entries()
			.filter(([key]) => ['name'].includes(key))
			.toArray()
	);

	if (searchParams.size > 0) {
		return `${host}/api/products?${searchParams.toString()}`;
	} else {
		return `${host}/api/products`;
	}
}

export type GetResponse = {
	data: {
		id: string;
		created: Date;
		updated?: Date;
		data: {
			name: string;
			list_item_references: {
				id: string;
				data: {
					list_reference: {
						id: string;
						data: {
							name: string;
						};
					};
				};
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
