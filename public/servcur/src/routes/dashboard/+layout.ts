import { API_ROUTES } from '$lib/api.js';

export async function load({ fetch }) {
	return {
		stream: fetch(API_ROUTES.system),
	};
}
