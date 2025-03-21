import { del, get, host, patch, post, type DataParams } from '..';
import type { Pagination } from '../types';

function productsUrl() {
	return `${host}/api/products`;
}

export type ProductsRequest = {
	data: {
		name: string;
	}[];
};

export type ProductsResponse = {
	pagination: Pagination;
	data: {
		id: string;
		data: {
			name: string;
			shopping_list_item_links: {
				id: string;
			}[];
		};
	}[];
};

export const products = {
	url: () => {
		return productsUrl();
	},

	get: (search?: URLSearchParams, params?: DataParams): Promise<ProductsResponse> => {
		return get(productsUrl() + (search && search.size > 0 ? `?${search.toString()}` : ''), params);
	},

	post: (body: ProductsRequest, params?: DataParams): Promise<void> => {
		return post(productsUrl(), body, params);
	}
};

function productUrl(id: string) {
	return `${host}/api/products/${id}`;
}

export type ProductRequest = {
	name?: string;
};

export type ProductResponse = {
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

export const product = {
	url: (id: string) => {
		return productUrl(id);
	},

	get: async (id: string, params?: DataParams): Promise<ProductResponse> => {
		const data = await get<ProductResponse>(productUrl(id), params);
		return {
			...data,
			created: new Date(data.created),
			updated: data.updated && new Date(data.updated)
		};
	},

	patch: (id: string, body: ProductRequest, params?: DataParams): Promise<void> => {
		return patch(productUrl(id), body, params);
	},

	delete: (id: string, params?: DataParams): Promise<void> => {
		return del(productUrl(id), params);
	}
};
