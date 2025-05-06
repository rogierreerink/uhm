import { del, get, host, patch, type DataParams, type DataResponse } from '..';

function url(id: string) {
	return `${host}/api/blocks/${id}`;
}

export type GetResponse = {
	id: string;
	ts_created: Date;
	ts_updated?: Date;
	data: {
		kind:
			| {
					type: 'markdown';
					id: string;
					data: {
						markdown: string;
						html: string;
					};
			  }
			| {
					type: 'recipe_collection';
					id: string;
			  };
	};
};

export type PatchRequest = {
	kind:
		| {
				type: 'markdown';
				id: string;
		  }
		| {
				type: 'recipe_collection';
				id: string;
		  };
};

export default {
	url: (id: string) => {
		return url(id);
	},

	get: async (id: string, params?: DataParams): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(id), params);

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

	patch: (id: string, body: PatchRequest, params?: DataParams): Promise<DataResponse<void>> => {
		return patch(url(id), body, params);
	},

	delete: (id: string, params?: DataParams): Promise<DataResponse<void>> => {
		return del(url(id), params);
	}
};
