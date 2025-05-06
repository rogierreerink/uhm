import { get, host, post, type DataParams, type DataResponse } from '..';

function url() {
	return `${host}/api/lists`;
}

export type GetResponse = {
	data: {
		id: string;
		ts_created: Date;
		ts_updated?: Date;
		data: {
			name: string;
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
	url: () => {
		return url();
	},

	get: async (params?: DataParams): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(), params);

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
