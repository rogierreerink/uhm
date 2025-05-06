import { del, get, host, patch, type DataParams, type DataResponse } from '../..';

function url(list_id: string, id: string) {
	return `${host}/api/lists/${list_id}/items/${id}`;
}

export type GetResponse = {
	id: string;
	ts_created: Date;
	ts_updated?: Date;
	data: {
		checked: boolean;
		kind:
			| {
					type: 'product';
					id: string;
					data: {
						name: string;
					};
			  }
			| {
					type: 'temporary';
					data: {
						name: string;
					};
			  };
	};
};

export type PatchRequest = {
	checked?: boolean;
};

export default {
	url: (list_id: string, id: string) => {
		return url(list_id, id);
	},

	get: async (
		list_id: string,
		id: string,
		params?: DataParams
	): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(list_id, id), params);

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
		list_id: string,
		id: string,
		body: PatchRequest,
		params?: DataParams
	): Promise<DataResponse<void>> => {
		return patch(url(list_id, id), body, params);
	},

	delete: (list_id: string, id: string, params?: DataParams): Promise<DataResponse<void>> => {
		return del(url(list_id, id), params);
	}
};
