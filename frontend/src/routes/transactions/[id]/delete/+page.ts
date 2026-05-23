import { redirect } from '@sveltejs/kit';
import { load_user } from '$lib/api';

export async function load() {
	const { user } = await load_user();

	if (!user) {
		throw redirect(302, '/login');
	}
}
