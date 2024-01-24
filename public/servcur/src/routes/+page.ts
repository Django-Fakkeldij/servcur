import { routes } from '$lib/routes';
import { redirect } from '@sveltejs/kit';

export async function load() {
	return redirect(307, routes.dashboard);
}
