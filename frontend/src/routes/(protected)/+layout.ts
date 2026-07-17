import { redirect } from '@sveltejs/kit';
import { load_user } from '$lib/api';
import { auth } from '$lib/stores/auth.svelte';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import type { LayoutLoad } from '../$types';

export const load: LayoutLoad = async () => {
	// FAST PATH: Client-side navigation, already logged in.
	if (browser && auth.isLoggedIn) {
		load_user().then(({ user }) => {
			if (!user) {
				auth.isLoggedIn = false;
				goto(resolve('/login'));
			}
		});

		return {};
	}

	// SLOW PATH: Initial hard-refresh or not logged in yet.
	const { user } = await load_user();

	if (!user) {
		throw redirect(302, '/login');
	}

	// Optional: Return the user so all protected pages can access it via $page.data.user
	return { user };
};
