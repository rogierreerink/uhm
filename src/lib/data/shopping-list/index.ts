import { del, get, host, patch, post, type DataParams } from '..';
import type { Pagination } from '../types';

function shoppingListItemsUrl() {
	return `${host}/api/shopping-list`;
}

export type ShoppingListItemsRequest = {
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

export type ShoppingListItemsResponse = {
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

export const shoppingListItems = {
	url: () => {
		return shoppingListItemsUrl();
	},

	get: (search?: URLSearchParams, params?: DataParams): Promise<ShoppingListItemsResponse> => {
		return get(
			shoppingListItemsUrl() + (search && search.size > 0 ? `?${search.toString()}` : ''),
			params
		);
	},

	post: (body: ShoppingListItemsRequest, params?: DataParams): Promise<void> => {
		return post(shoppingListItemsUrl(), body, params);
	}
};

function shoppingListItemUrl(id: string) {
	return `${host}/api/shopping-list/${id}`;
}

export type ShoppingListItemRequest = {
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

export type ShoppingListItemResponse = {
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

export const shoppingListItem = {
	url: (id: string) => {
		return shoppingListItemUrl(id);
	},

	get: async (id: string, params?: DataParams): Promise<ShoppingListItemResponse> => {
		const data = await get<ShoppingListItemResponse>(shoppingListItemUrl(id), params);
		return {
			...data,
			created: new Date(data.created),
			updated: data.updated && new Date(data.updated)
		};
	},

	patch: (id: string, body: ShoppingListItemRequest, params?: DataParams): Promise<void> => {
		return patch(shoppingListItemUrl(id), body, params);
	},

	delete: (id: string, params?: DataParams): Promise<void> => {
		return del(shoppingListItemUrl(id), params);
	}
};
