export const host = 'http://localhost:3002';

export type DataParams = {
	fetcher?: typeof fetch;
};

export async function get<R>(url: string, params?: DataParams): Promise<R> {
	return await (
		await (params?.fetcher || fetch)(url, {
			method: 'GET',
			headers: { accept: 'application/json' }
		})
	).json();
}

export async function post<B>(url: string, body: B, params?: DataParams): Promise<void> {
	await (params?.fetcher || fetch)(url, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(body)
	});
}

export async function patch<B>(url: string, body: B, params?: DataParams): Promise<void> {
	await (params?.fetcher || fetch)(url, {
		method: 'PATCH',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(body)
	});
}

export async function del(url: string, params?: DataParams): Promise<void> {
	await (params?.fetcher || fetch)(url, {
		method: 'DELETE'
	});
}
