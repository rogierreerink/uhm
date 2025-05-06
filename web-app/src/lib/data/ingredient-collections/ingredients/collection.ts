import { get, host, post, type DataParams, type DataResponse } from '../..';

function url(collectionId: string) {
	return `${host}/api/ingredient-collections/${collectionId}/ingredients`;
}

export type GetResponse = {
	data: {
		id: string;
		ts_created: Date;
		ts_updated?: Date;
		data: {
			product: {
				id: string;
				data: {
					name: string;
				};
			};
		};
	}[];
};

export type PostRequest = {
	data: {
		product: {
			id: string;
		};
	}[];
};

export type PostResponse = {
	data: {
		id: string;
		data: {
			product: {
				id: string;
			};
		};
	}[];
};

export default {
	url: (collectionId: string) => {
		return url(collectionId);
	},

	get: async (collectionId: string, params?: DataParams): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(collectionId), params);

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

	post: (
		list_id: string,
		body: PostRequest,
		params?: DataParams
	): Promise<DataResponse<PostResponse>> => {
		return post(url(list_id), body, params);
	}
};
