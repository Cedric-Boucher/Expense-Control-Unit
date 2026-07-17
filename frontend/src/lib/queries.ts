import { createQuery, createMutation, useQueryClient } from '@tanstack/svelte-query';
import {
	getTransactions,
	getCategories,
	getTags,
	createTransaction,
	updateTransaction,
	deleteTransaction,
	createCategory,
	updateCategory,
	deleteCategory
} from '$lib/api';
import type { Transaction, Category, Tag, NewTransaction, NewCategory } from '$lib/types';

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

// --- Transaction Mutations ---

export function useCreateTransaction() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: (payload: NewTransaction) => createTransaction(payload),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['appData'] });
		}
	}));
}

export function useUpdateTransaction() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		// mutationFn only takes one variable, so we pass an object containing both id and data
		mutationFn: ({ id, data }: { id: string; data: NewTransaction }) =>
			updateTransaction(id, data),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['appData'] });
		}
	}));
}

export function useDeleteTransaction() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: (id: string) => deleteTransaction(id),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['appData'] });
		}
	}));
}

// --- Category Mutations ---

export function useCreateCategory() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: (payload: NewCategory) => createCategory(payload),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['appData'] });
		}
	}));
}

export function useUpdateCategory() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: ({ id, data }: { id: string; data: NewCategory }) => updateCategory(id, data),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['appData'] });
		}
	}));
}

export function useDeleteCategory() {
	const queryClient = useQueryClient();

	return createMutation(() => ({
		mutationFn: (id: string) => deleteCategory(id),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['appData'] });
		}
	}));
}
