import { redirect } from '@sveltejs/kit';
import { load_user } from '$lib/api';

export async function load({ url }) {
	// ignore background asset requests
	if (url.pathname.includes('.')) {
		return;
	}
	console.log('Catch-all route triggered for:', url.pathname);
	const { user } = await load_user();
	if (user) {
		throw redirect(302, '/transactions');
	} else {
		throw redirect(302, '/login');
	}
}
