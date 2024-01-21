import { API_ROUTES } from '$lib/api.js';

export async function load({ fetch }) {
	return {
		containers_stream: fetch(API_ROUTES.images),
	};
}
