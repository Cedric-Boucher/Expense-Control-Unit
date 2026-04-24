<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import type { Transaction, Category } from '$lib/types';
	import TransactionCard from '$lib/components/TransactionCard.svelte';
	import { SvelteSet } from 'svelte/reactivity';

	let { data }: { data: { transactions: Transaction[]; categories: Category[] } } = $props();

	let categoryMap = $derived(new Map(data.categories.map((c) => [c.id, c])));

	let categoriesWithPath = $derived(
		data.categories
			.map((c) => {
				let path = c.name;
				let curr = c;
				while (curr.parent_id && categoryMap.has(curr.parent_id)) {
					curr = categoryMap.get(curr.parent_id)!;
					path = curr.name + ' / ' + path;
				}
				return { ...c, pathName: path };
			})
			.sort((a, b) => a.pathName.localeCompare(b.pathName))
	);

	let selectedCategoryIds = $state<Set<number>>(
		untrack(() => new Set(categoriesWithPath.map((c) => c.id)))
	);

	let isFilterActive = $derived(selectedCategoryIds.size < categoriesWithPath.length);

	let filteredTransactions = $derived(
		data.transactions.filter((tx) => selectedCategoryIds.has(tx.category.id))
	);

	const BATCH_SIZE = 100;
	const ESTIMATED_ITEM_HEIGHT = 80;

	let limit = $state(BATCH_SIZE);

	function loadMore() {
		if (limit < filteredTransactions.length) {
			limit += BATCH_SIZE;
			setTimeout(loadMore, 0);
		}
	}

	$effect(() => {
		if (filteredTransactions) {
			untrack(() => {
				limit = BATCH_SIZE;
				if (typeof window !== 'undefined') window.scrollTo(0, 0);
				setTimeout(loadMore, 0);
			});
		}
	});

	let visibleTransactions = $derived(filteredTransactions.slice(0, limit));
	let remainingCount = $derived(Math.max(0, filteredTransactions.length - limit));
	let phantomHeight = $derived(remainingCount * ESTIMATED_ITEM_HEIGHT);

	let isFilterOpen = $state(false);
	let filterContainer: HTMLDivElement | undefined = $state();
	let searchQuery = $state('');

	let visibleCategories = $derived(
		categoriesWithPath.filter((cat) =>
			cat.pathName.toLowerCase().includes(searchQuery.toLowerCase())
		)
	);

	function toggleCategory(id: number) {
		const newSet = new SvelteSet(selectedCategoryIds);
		if (newSet.has(id)) {
			newSet.delete(id);
		} else {
			newSet.add(id);
		}
		selectedCategoryIds = newSet;
	}

	function selectAll() {
		selectedCategoryIds = new Set(categoriesWithPath.map((c) => c.id));
	}

	function clearAll() {
		selectedCategoryIds = new Set();
	}

	$effect(() => {
		function handleClickOutside(event: MouseEvent) {
			if (filterContainer && !filterContainer.contains(event.target as Node)) {
				isFilterOpen = false;
			}
		}
		document.addEventListener('click', handleClickOutside);

		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
</script>

<h1 class="text-2xl font-bold mb-4">Transactions</h1>

<div class="mb-6 flex items-center gap-4">
	<button
		class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
		onclick={() => goto('/transactions/new')}
	>
		+ New Transaction
	</button>

	<div class="relative" bind:this={filterContainer}>
		<button
			type="button"
			class="px-4 py-2 rounded flex items-center gap-2 transition-colors
            {isFilterActive
				? 'bg-amber-200 text-amber-900 hover:bg-amber-300 dark:bg-amber-700 dark:text-amber-100 dark:hover:bg-amber-800'
				: 'bg-gray-200 text-gray-900 hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600'}"
			onclick={() => (isFilterOpen = !isFilterOpen)}
		>
			<span class="font-medium">
				{isFilterActive ? 'Filter Categories (Active)' : 'Filter Categories'}
			</span>
			<svg
				class="w-4 h-4"
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M19 9l-7 7-7-7"
				/>
			</svg>
		</button>

		{#if isFilterOpen}
			<div
				class="absolute left-0 mt-2 w-64 bg-gray-100 dark:bg-gray-800 p-4 rounded shadow-lg z-10"
			>
				<input
					type="text"
					placeholder="Search categories..."
					class="w-full p-2 mb-3 border rounded dark:bg-gray-700 dark:text-white"
					bind:value={searchQuery}
				/>

				<div class="flex flex-col gap-2 max-h-64 overflow-auto">
					{#if visibleCategories.length > 0}
						{#each visibleCategories as cat (cat.id)}
							<label class="flex items-center space-x-2">
								<input
									type="checkbox"
									checked={selectedCategoryIds.has(cat.id)}
									onchange={() => toggleCategory(cat.id)}
								/>
								<span>{cat.pathName}</span>
							</label>
						{/each}
					{:else}
						<p class="text-sm text-gray-500 italic">No categories found</p>
					{/if}
				</div>

				<div class="flex justify-between mt-3">
					<button
						type="button"
						class="px-3 py-1 rounded bg-blue-500 text-white hover:bg-blue-600"
						onclick={selectAll}
					>
						Select All
					</button>
					<button
						type="button"
						class="px-3 py-1 rounded bg-gray-400 text-white hover:bg-gray-500"
						onclick={clearAll}
					>
						Deselect All
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>

{#if visibleTransactions.length}
	<ul class="space-y-4">
		{#each visibleTransactions as tx (tx.id)}
			<TransactionCard transaction={tx} allCategories={data.categories} showActions={true} />
		{/each}

		<div style="height: {phantomHeight}px; width: 100%"></div>
	</ul>

	{#if limit < filteredTransactions.length}
		<p class="text-center text-xs text-gray-400 mt-2">Loading rest of data...</p>
	{:else}
		<p class="text-center text-xs text-gray-400 mt-2 mb-8">
			Showing all {filteredTransactions.length} transactions
		</p>
	{/if}
{:else}
	<p class="text-gray-500 italic">No transactions match the selected categories.</p>
{/if}
