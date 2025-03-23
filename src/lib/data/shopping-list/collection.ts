import { get, host, post, type DataParams, type DataResponse } from '..';
import type { Pagination } from '../types';

function url() {
	return `${host}/api/shopping-list`;
}

export type GetResponse = {
	pagination: Pagination;
	data: {
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
	}[];
};

export type PostRequest = {
	data: {
		inCart?: boolean;
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
			  };
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
