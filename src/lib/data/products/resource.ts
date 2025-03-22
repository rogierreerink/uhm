import { del, get, host, patch, type DataParams } from '..';

function url(id: string) {
	return `${host}/api/products/${id}`;
}

export type GetResponse = {
	id: string;
	created: Date;
	updated?: Date;
	data: {
		name: string;
		shopping_list_item_links: {
			id: string;
		}[];
	};
};

export type PatchRequest = {
	name?: string;
};

export default {
	url: (id: string) => {
		return url(id);
	},

	get: async (id: string, params?: DataParams): Promise<GetResponse> => {
		const data = await get<GetResponse>(url(id), params);
		return {
			...data,
			created: new Date(data.created),
			updated: data.updated && new Date(data.updated)
		};
	},

	patch: (id: string, body: PatchRequest, params?: DataParams): Promise<void> => {
		return patch(url(id), body, params);
	},

	delete: (id: string, params?: DataParams): Promise<void> => {
		return del(url(id), params);
	}
};
