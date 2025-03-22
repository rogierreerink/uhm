import { get, host, post, type DataParams } from '..';
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

	get: (params?: DataParams): Promise<GetResponse> => {
		return get(url(), params);
	},

	post: (body: PostRequest, params?: DataParams): Promise<PostResponse> => {
		return post(url(), body, params);
	}
};
