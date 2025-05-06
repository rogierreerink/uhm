import { get, host, post, type DataParams, type DataResponse } from '..';

function url(searchParams: URLSearchParams = new URLSearchParams()) {
	searchParams = new URLSearchParams(
		[...searchParams.entries()].filter(([key]) => ['type'].includes(key))
	);

	if (searchParams.size > 0) {
		return `${host}/api/pages?${searchParams.toString()}`;
	} else {
		return `${host}/api/pages`;
	}
}

export type PageType = 'recipe';

export type GetResponse = {
	data: {
		id: string;
		ts_created: Date;
		ts_updated?: Date;
		data: {
			type: PageType;
			name: string;
			blocks: {
				id: string;
			}[];
		};
	}[];
};

export type PostRequest = {
	data: {
		type: PageType;
		name: string;
		blocks: {
			id: string;
		}[];
	}[];
};

export type PostResponse = {
	data: {
		type: PageType;
		id: string;
		blocks: {
			id: string;
		}[];
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
					created: new Date(item.ts_created),
					updated: item.ts_updated && new Date(item.ts_updated)
				}))
			}
		};
	},

	post: (body: PostRequest, params?: DataParams): Promise<DataResponse<PostResponse>> => {
		return post(url(), body, params);
	}
};
