import { API_URL } from '$lib/api.js';

export async function load({ fetch }) {
	return {
		containers: (await (await fetch(API_URL + '/containers')).json()) as Record<string, unknown>,
	};
}
