export const host = 'http://192.168.1.228:3002';

export type DataParams = {
	fetcher?: typeof fetch;
};

export type DataResponse<R> =
	| { ok: true; data: R; response: Response }
	| { ok: false; response: Response };

export async function get<R>(url: string, params?: DataParams): Promise<DataResponse<R>> {
	const response = await (params?.fetcher || fetch)(url, {
		method: 'GET',
		headers: { accept: 'application/json' }
	});

	if (!response.ok)
		return {
			ok: false,
			response
		};

	return {
		ok: true,
		data: await response.json(),
		response
	};
}

export async function post<B, R>(
	url: string,
	body: B,
	params?: DataParams
): Promise<DataResponse<R>> {
	const response = await (params?.fetcher || fetch)(url, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(body)
	});

	if (!response.ok)
		return {
			ok: false,
			response
		};

	return {
		ok: true,
		data: await response.json(),
		response
	};
}

export async function patch<B>(
	url: string,
	body: B,
	params?: DataParams
): Promise<DataResponse<void>> {
	const response = await (params?.fetcher || fetch)(url, {
		method: 'PATCH',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(body)
	});

	if (!response.ok)
		return {
			ok: false,
			response
		};

	return {
		ok: true,
		data: undefined,
		response
	};
}

export async function del(url: string, params?: DataParams): Promise<DataResponse<void>> {
	const response = await (params?.fetcher || fetch)(url, {
		method: 'DELETE'
	});

	if (!response.ok)
		return {
			ok: false,
			response
		};

	return {
		ok: true,
		data: undefined,
		response
	};
}
