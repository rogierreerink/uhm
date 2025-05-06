export type CollectionRequest<D> = {
	data: D[];
};

export type CollectionResponse<I, D> = {
	pagination?: Pagination;
	data: ResourceResponse<I, D>[];
};

export type ResourceResponse<I, D> = {
	id: I;
	data: D;
};

export type Pagination = {
	skip: number;
	take: number;
	total: number;
};
