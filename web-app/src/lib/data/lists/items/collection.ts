import { get, host, post, type DataParams, type DataResponse } from '../..';

function url(list_id: string) {
	return `${host}/api/lists/${list_id}/items`;
}

export type GetResponse = {
	data: {
		id: string;
		ts_created: Date;
		ts_updated?: Date;
		data: {
			checked: boolean;
			kind:
				| {
						type: 'ingredient';
						id: string;
				  }
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
	}[];
};

export type PostRequest = {
	data: {
		kind:
			| {
					type: 'ingredient';
					id: string;
			  }
			| {
					type: 'product';
					id: string;
			  }
			| {
					type: 'temporary';
					data: {
						name: string;
					};
			  };
	}[];
};

export type PostResponse = {
	data: {
		id: string;
	}[];
};

export default {
	url: (list_id: string) => {
		return url(list_id);
	},

	get: async (list_id: string, params?: DataParams): Promise<DataResponse<GetResponse>> => {
		const response = await get<GetResponse>(url(list_id), params);

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
