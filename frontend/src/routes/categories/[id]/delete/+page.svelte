<script lang="ts">
	import { page } from '$app/state';
	import { getCategory, deleteCategory, getCategoryTransactions, getCategories } from '$lib/api';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import type { Category, Transaction } from '$lib/types';
	import CategoryCard from '$lib/components/CategoryCard.svelte';
	import TransactionCard from '$lib/components/TransactionCard.svelte';

	let category: Category | null = null;
	let transactions: Transaction[] = [];
	let childCategories: Category[] = [];
	let error = '';
	let loading = true;

	const id = page.params.id;
	const numericId = Number(id);
	const redirectTo = page.url.searchParams.get('redirectTo') ?? '/categories';

	onMount(async () => {
		if (!id) {
			goto(redirectTo);
			return;
		}

		try {
			// Fetch everything concurrently for better performance
			const [fetchedCategory, fetchedTransactions, allCategories] = await Promise.all([
				getCategory(id),
				getCategoryTransactions(id),
				getCategories()
			]);

			category = fetchedCategory;
			transactions = fetchedTransactions;

			// Find all categories that have this one as a direct parent
			childCategories = allCategories.filter((c) => c.parent_id === numericId);
		} catch (e) {
			error = 'Failed to load category data.';
			console.error(e);
		} finally {
			loading = false;
		}
	});

	async function confirmDelete() {
		try {
			if (id) {
				await deleteCategory(id);
			}
			goto(redirectTo);
		} catch (e) {
			error = 'Failed to delete category.';
			console.error(e);
		}
	}

	function cancel() {
		goto(redirectTo);
	}
</script>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="text-red-600">{error}</p>
{:else if category}
	<h1 class="text-2xl font-bold mb-4">Delete Category</h1>

	<p class="mb-2">Are you sure you want to delete the following category?</p>
	<CategoryCard {category} showActions={false} />

	{#if childCategories.length > 0}
		<p class="mt-4 text-red-600">
			This category is a parent to the following child categor{childCategories.length > 1
				? 'ies'
				: 'y'}:
		</p>
		<ul class="space-y-2 mt-2">
			{#each childCategories as child (child.id)}
				<CategoryCard category={child} showActions={true} />
			{/each}
		</ul>
		<p class="mt-4 mb-6 text-sm text-gray-700 dark:text-gray-300">
			Please edit these child categories to use a different parent before deleting this
			category.
		</p>
	{/if}

	{#if transactions.length > 0}
		<p class="mt-4 text-red-600">
			This category is currently used by the following transaction{transactions.length > 1
				? 's'
				: ''}:
		</p>
		<ul class="space-y-2 mt-2">
			{#each transactions as tx (tx.id)}
				<TransactionCard transaction={tx} showActions={true} />
			{/each}
		</ul>

		<p class="mt-4 text-sm text-gray-700 dark:text-gray-300">
			Please update or delete {transactions.length > 1
				? 'these transactions'
				: 'this transaction'} before deleting this category.
		</p>
		<div class="mt-4">
			<button
				on:click={cancel}
				class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
			>
				Cancel
			</button>
		</div>
	{:else}
		<div class="flex space-x-4 mt-4">
			<button
				on:click={confirmDelete}
				class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
			>
				Yes, Delete
			</button>
			<button
				on:click={cancel}
				class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
			>
				Cancel
			</button>
		</div>
	{/if}
{:else}
	<p class="text-gray-500 italic">Category not found.</p>
{/if}
