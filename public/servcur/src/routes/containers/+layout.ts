import { API_URL } from '$lib/api.js';

export async function load({ fetch }) {
	return {
		containers_stream: fetch(API_URL + '/containers'),
	};
}
