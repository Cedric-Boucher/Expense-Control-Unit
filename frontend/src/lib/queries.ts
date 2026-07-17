import { createQuery } from '@tanstack/svelte-query';
import { getTransactions, getCategories, getTags } from '$lib/api';
import type { Transaction, Category, Tag } from '$lib/types';

export interface AppData {
	transactions: Transaction[];
	categories: Category[];
	tags: Tag[];
}

export function useAppData() {
	return createQuery<AppData, Error>(() => ({
		queryKey: ['appData'],
		queryFn: async (): Promise<AppData> => {
			const [transactions, categories, tags] = await Promise.all([
				getTransactions(),
				getCategories(),
				getTags()
			]);

			return { transactions, categories, tags };
		}
	}));
}
