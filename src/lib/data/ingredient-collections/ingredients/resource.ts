import { del, get, host, patch, type DataParams, type DataResponse } from '../..';

function url(collectionId: string, id: string) {
	return `${host}/api/ingredient-collections/${collectionId}/ingredients/${id}`;
}

export type GetResponse = {
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
};

export type PatchRequest = {
	product: {
		id: string;
	};
};

export default {
	url: (collectionId: string, id: string) => {
		return url(collectionId, id);
	},

	get: async (
		collectionId: string,
		id: string,
		params?: DataParams
	): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(collectionId, id), params);

		if (!response.ok) {
			return response;
		}

		return {
			...response,
			data: {
				...response.data,
				ts_created: new Date(response.data.ts_created),
				ts_updated: response.data.ts_updated && new Date(response.data.ts_updated)
			}
		};
	},

	patch: (
		collectionId: string,
		id: string,
		body: PatchRequest,
		params?: DataParams
	): Promise<DataResponse<void>> => {
		return patch(url(collectionId, id), body, params);
	},

	delete: (collectionId: string, id: string, params?: DataParams): Promise<DataResponse<void>> => {
		return del(url(collectionId, id), params);
	}
};
