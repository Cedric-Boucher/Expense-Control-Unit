<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { useAppData, useDeleteCategory } from '$lib/queries';
	import CategoryCard from '$lib/components/CategoryCard.svelte';
	import TransactionCard from '$lib/components/TransactionCard.svelte';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';
	import AsyncButton from '$lib/components/AsyncButton.svelte';

	const appData = useAppData();
	const deleteCategoryMutation = useDeleteCategory();

	let error = $state('');

	let id = $derived(page.params.id);
	let numericId = $derived(Number(id));
	let redirectTo = $derived(
		(page.url.searchParams.get('redirectTo') ?? '/categories') as Pathname
	);

	let category = $derived(appData.data?.categories.find((c) => c.id === numericId) ?? null);

	// Find all categories that have this one as a direct parent
	let childCategories = $derived(
		appData.data?.categories.filter((c) => c.parent_id === numericId) ?? []
	);

	// Find any transactions associated with this category
	let transactions = $derived(
		appData.data?.transactions.filter((t) => t.category.id === numericId) ?? []
	);

	$effect(() => {
		if (!id) {
			if (redirectTo) goto(resolve(redirectTo));
		}
	});

	async function confirmDelete() {
		error = '';
		try {
			if (id) {
				await deleteCategoryMutation.mutateAsync(id);
			}
			await goto(resolve(redirectTo));
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete category.';
			console.error(e);
		}
	}

	async function cancel() {
		await goto(resolve(redirectTo));
	}
</script>

{#if appData.isPending}
	<p>Loading...</p>
{:else if appData.isError}
	<p class="text-red-600">Failed to load category data.</p>
{:else if category}
	<h1 class="text-2xl font-bold mb-4">Delete Category</h1>

	{#if error}
		<p class="text-red-600 mb-4">{error}</p>
	{/if}

	<p class="mb-2">Are you sure you want to delete the following category?</p>
	<CategoryCard {category} allCategories={appData.data?.categories ?? []} showActions={false} />

	{#if childCategories.length > 0}
		<p class="mt-4 text-red-600">
			This category is a parent to the following child categor{childCategories.length > 1
				? 'ies'
				: 'y'}:
		</p>
		<ul class="space-y-2 mt-2">
			{#each childCategories as child (child.id)}
				<CategoryCard
					category={child}
					allCategories={appData.data?.categories ?? []}
					showActions={true}
				/>
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
	{/if}
	<div class="flex space-x-4 mt-4">
		{#if childCategories.length == 0 && transactions.length == 0}
			<AsyncButton
				action={confirmDelete}
				class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
			>
				Yes, Delete
			</AsyncButton>
		{/if}
		<AsyncButton
			action={cancel}
			class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
		>
			Cancel
		</AsyncButton>
	</div>
{:else}
	<p class="text-gray-500 italic">Category not found.</p>
{/if}
