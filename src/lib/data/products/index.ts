import { del, get, host, patch, post, type DataParams } from '..';
import type { Pagination } from '../types';

export function productsUrl() {
	return `${host}/api/products`;
}

export type ProductsSearchParams = {
	name?: string;
};

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

	get: (search?: ProductsSearchParams, params?: DataParams): Promise<ProductsResponse> => {
		return get(
			productsUrl() + (search ? `?${new URLSearchParams(search).toString()}` : ''),
			params
		);
	},

	post: (body: ProductsRequest, params?: DataParams): Promise<void> => {
		return post(productsUrl(), body, params);
	}
};

export function productUrl(id: string) {
	return `${host}/api/products/${id}`;
}

export type ProductRequest = {
	data: {
		name?: string;
	};
};

export type ProductResponse = {
	id: string;
	data: {
		name: string;
	};
};

export const product = {
	url: (id: string) => {
		return productUrl(id);
	},

	get: (id: string, params?: DataParams): Promise<ProductResponse> => {
		return get(productUrl(id), params);
	},

	patch: (id: string, body: ProductRequest, params?: DataParams): Promise<void> => {
		return patch(productUrl(id), body, params);
	},

	delete: (id: string, params?: DataParams): Promise<void> => {
		return del(productUrl(id), params);
	}
};
