import { redirect } from '@sveltejs/kit';
import { getTransactions, getCategories, getTags, load_user } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
	const { user } = await load_user();

	if (!user) {
		throw redirect(302, '/login');
	}

	const [transactions, categories, tags] = await Promise.all([
		getTransactions(),
		getCategories(),
		getTags()
	]);

	return { transactions, categories, tags };
};
