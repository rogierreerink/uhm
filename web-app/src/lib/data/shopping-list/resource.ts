import { del, get, host, patch, type DataParams, type DataResponse } from '..';

function url(id: string) {
	return `${host}/api/shopping-list/${id}`;
}

export type GetResponse = {
	id: string;
	created: Date;
	updated?: Date;
	data: {
		inCart: boolean;
		source:
			| {
					type: 'temporary';
					data: {
						name: string;
					};
			  }
			| {
					type: 'product';
					id: string;
					data: {
						name: string;
					};
			  };
	};
};

export type PatchRequest = {
	inCart?: boolean;
	source?:
		| {
				type: 'temporary';
				data: {
					name?: string;
				};
		  }
		| {
				type: 'product';
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
				created: new Date(response.data.created),
				updated: response.data.updated && new Date(response.data.updated)
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
