import { getTransactions, getCategories, getTags } from '$lib/api';
import type { PageLoad } from '../../$types';

export const load: PageLoad = async () => {
	const [transactions, categories, tags] = await Promise.all([
		getTransactions(),
		getCategories(),
		getTags()
	]);

	return { transactions, categories, tags };
};
