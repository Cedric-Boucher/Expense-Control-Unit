<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import TransactionCard from '$lib/components/TransactionCard.svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';
	import { useAppData } from '$lib/queries';
	import { createPersistedState } from '$lib/persistedState.svelte';

	const appQuery = useAppData();

	// Use null to represent "first visit / no saved filters" OR "everything is selected"
	const persistentFilters = createPersistedState<{
		categories: number[] | null;
		tags: number[] | null;
	}>('transaction_filters', { categories: null, tags: null });

	// Safely unwrap data with fallbacks for the initial loading state
	const rawCategories = $derived(appQuery.data?.categories ?? []);
	const rawTags = $derived(appQuery.data?.tags ?? []);
	const rawTransactions = $derived(appQuery.data?.transactions ?? []);

	// --- Category Logic ---
	let categoryMap = $derived(new Map(rawCategories.map((c) => [c.id, c])));

	let categoriesWithPath = $derived(
		rawCategories
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

	// Initialize Set using persisted array if available
	let selectedCategoryIds = new SvelteSet<number>(persistentFilters.value.categories ?? []);

	let isCategoryFilterActive = $derived(
		selectedCategoryIds.size > 0 && selectedCategoryIds.size < categoriesWithPath.length
	);
	let isCategoryFilterOpen = $state(false);
	let categoryFilterContainer: HTMLDivElement | undefined = $state();
	let categorySearchQuery = $state('');

	let visibleCategories = $derived(
		categoriesWithPath.filter((cat) =>
			cat.pathName.toLowerCase().includes(categorySearchQuery.toLowerCase())
		)
	);

	function toggleCategory(id: number) {
		if (selectedCategoryIds.has(id)) {
			selectedCategoryIds.delete(id);
		} else {
			selectedCategoryIds.add(id);
		}
	}

	function selectAllCategories() {
		selectedCategoryIds.clear();
		categoriesWithPath.forEach((c) => selectedCategoryIds.add(c.id));
	}

	function clearAllCategories() {
		selectedCategoryIds.clear();
	}

	// --- Tag Logic ---
	let sortedTags = $derived([...rawTags].sort((a, b) => a.name.localeCompare(b.name)));

	// Initialize Set using persisted array if available
	let selectedTagIds = new SvelteSet<number>(persistentFilters.value.tags ?? []);

	let isTagFilterActive = $derived(
		selectedTagIds.size > 0 && selectedTagIds.size < sortedTags.length
	);
	let isTagFilterOpen = $state(false);
	let tagFilterContainer: HTMLDivElement | undefined = $state();
	let tagSearchQuery = $state('');

	let visibleTags = $derived(
		sortedTags.filter((tag) => tag.name.toLowerCase().includes(tagSearchQuery.toLowerCase()))
	);

	function toggleTag(id: number) {
		if (selectedTagIds.has(id)) {
			selectedTagIds.delete(id);
		} else {
			selectedTagIds.add(id);
		}
	}

	function selectAllTags() {
		selectedTagIds.clear();
		sortedTags.forEach((t) => selectedTagIds.add(t.id));
	}

	function clearAllTags() {
		selectedTagIds.clear();
	}

	// --- State Initialization & Syncing ---
	let filtersInitialized = $state(false);

	let knownCategoryIds = new SvelteSet<number>();
	let isKnownCategoriesInitialized = false;

	let knownTagIds = new SvelteSet<number>();
	let isKnownTagsInitialized = false;

	// 1. Initial Load: Use $effect.pre so it sets up before the DOM paints (prevents flickering)
	$effect.pre(() => {
		if (appQuery.data && !filtersInitialized) {
			untrack(() => {
				// If null (first visit or "all selected" state), apply the "Select All"
				if (persistentFilters.value.categories === null) {
					selectAllCategories();
				}

				if (persistentFilters.value.tags === null) {
					selectAllTags();
				}

				filtersInitialized = true;
			});
		}
	});

	// 2. Sync Set mutations back to the persistent store
	$effect(() => {
		if (filtersInitialized) {
			const catArray = Array.from(selectedCategoryIds);
			const tagArray = Array.from(selectedTagIds);

			// If everything is selected, save `null` instead of an array of all IDs.
			// This ensures cross-navigation additions default to "select all".
			persistentFilters.value = {
				categories: catArray.length === categoriesWithPath.length ? null : catArray,
				tags: tagArray.length === sortedTags.length ? null : tagArray
			};
		}
	});

	// 3. Auto-select new categories if everything was selected previously (in-place modal additions)
	$effect(() => {
		// Read derived dependencies BEFORE any early return so Svelte always tracks them
		const currentCatIds = categoriesWithPath.map((c) => c.id);

		if (!filtersInitialized) return;

		untrack(() => {
			if (isKnownCategoriesInitialized) {
				const newIds = currentCatIds.filter((id) => !knownCategoryIds.has(id));
				if (newIds.length > 0) {
					let allKnownSelected = true;
					for (const id of knownCategoryIds) {
						if (!selectedCategoryIds.has(id)) {
							allKnownSelected = false;
							break;
						}
					}
					if (allKnownSelected) {
						newIds.forEach((id) => selectedCategoryIds.add(id));
					}
				}
			}

			knownCategoryIds.clear();
			currentCatIds.forEach((id) => knownCategoryIds.add(id));
			isKnownCategoriesInitialized = true;
		});
	});

	// 4. Auto-select new tags if everything was selected previously (in-place modal additions)
	$effect(() => {
		// Read derived dependencies BEFORE any early return so Svelte always tracks them
		const currentTagIds = sortedTags.map((t) => t.id);

		if (!filtersInitialized) return;

		untrack(() => {
			if (isKnownTagsInitialized) {
				const newIds = currentTagIds.filter((id) => !knownTagIds.has(id));
				if (newIds.length > 0) {
					let allKnownSelected = true;
					for (const id of knownTagIds) {
						if (!selectedTagIds.has(id)) {
							allKnownSelected = false;
							break;
						}
					}
					if (allKnownSelected) {
						newIds.forEach((id) => selectedTagIds.add(id));
					}
				}
			}

			knownTagIds.clear();
			currentTagIds.forEach((id) => knownTagIds.add(id));
			isKnownTagsInitialized = true;
		});
	});

	// --- Filtering & Pagination ---
	let filteredTransactions = $derived(
		rawTransactions.filter((tx) => {
			const catMatch = selectedCategoryIds.has(tx.category.id);
			const tagMatch =
				!isTagFilterActive ||
				(tx.tags && tx.tags.some((tag) => selectedTagIds.has(tag.id)));
			return catMatch && tagMatch;
		})
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

	$effect(() => {
		function handleClickOutside(event: MouseEvent) {
			if (
				categoryFilterContainer &&
				!categoryFilterContainer.contains(event.target as Node)
			) {
				isCategoryFilterOpen = false;
			}
			if (tagFilterContainer && !tagFilterContainer.contains(event.target as Node)) {
				isTagFilterOpen = false;
			}
		}
		document.addEventListener('click', handleClickOutside);

		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
</script>

<h1 class="text-2xl font-bold mb-4">Transactions</h1>

{#if appQuery.isPending}
	<div class="py-12 text-center text-gray-500">
		<p>Loading your transactions...</p>
	</div>
{:else if appQuery.isError}
	<div class="py-12 text-center text-red-500">
		<p>Error loading data: {appQuery.error.message}</p>
	</div>
{:else}
	<div class="mb-6 flex flex-wrap items-center gap-4">
		<AsyncButton
			class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 whitespace-nowrap"
			action={async () => await goto(resolve('/transactions/new'))}
		>
			+ New Transaction
		</AsyncButton>

		<div class="relative" bind:this={categoryFilterContainer}>
			<button
				type="button"
				class="px-4 py-2 rounded flex items-center gap-2 transition-colors
                {isCategoryFilterActive
					? 'bg-amber-200 text-amber-900 hover:bg-amber-300 dark:bg-amber-700 dark:text-amber-100 dark:hover:bg-amber-800'
					: 'bg-gray-200 text-gray-900 hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600'}"
				onclick={() => (isCategoryFilterOpen = !isCategoryFilterOpen)}
			>
				<span class="font-medium">
					{isCategoryFilterActive ? 'Filter Categories (Active)' : 'Filter Categories'}
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

			{#if isCategoryFilterOpen}
				<div
					class="absolute left-0 mt-2 w-64 bg-gray-100 dark:bg-gray-800 p-4 rounded shadow-lg z-20"
				>
					<input
						type="text"
						placeholder="Search categories..."
						class="w-full p-2 mb-3 border rounded dark:bg-gray-700 dark:text-white"
						bind:value={categorySearchQuery}
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
							onclick={selectAllCategories}>Select All</button
						>
						<button
							type="button"
							class="px-3 py-1 rounded bg-gray-400 text-white hover:bg-gray-500"
							onclick={clearAllCategories}>Deselect All</button
						>
					</div>
				</div>
			{/if}
		</div>

		<div class="relative" bind:this={tagFilterContainer}>
			<button
				type="button"
				class="px-4 py-2 rounded flex items-center gap-2 transition-colors
                {isTagFilterActive
					? 'bg-amber-200 text-amber-900 hover:bg-amber-300 dark:bg-amber-700 dark:text-amber-100 dark:hover:bg-amber-800'
					: 'bg-gray-200 text-gray-900 hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600'}"
				onclick={() => (isTagFilterOpen = !isTagFilterOpen)}
			>
				<span class="font-medium">
					{isTagFilterActive ? 'Filter Tags (Active)' : 'Filter Tags'}
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

			{#if isTagFilterOpen}
				<div
					class="absolute left-0 mt-2 w-64 bg-gray-100 dark:bg-gray-800 p-4 rounded shadow-lg z-20"
				>
					<input
						type="text"
						placeholder="Search tags..."
						class="w-full p-2 mb-3 border rounded dark:bg-gray-700 dark:text-white"
						bind:value={tagSearchQuery}
					/>

					<div class="flex flex-col gap-2 max-h-64 overflow-auto">
						{#if visibleTags.length > 0}
							{#each visibleTags as tag (tag.id)}
								<label class="flex items-center space-x-2">
									<input
										type="checkbox"
										checked={selectedTagIds.has(tag.id)}
										onchange={() => toggleTag(tag.id)}
									/>
									<span>{tag.name}</span>
								</label>
							{/each}
						{:else}
							<p class="text-sm text-gray-500 italic">No tags found</p>
						{/if}
					</div>

					<div class="flex justify-between mt-3">
						<button
							type="button"
							class="px-3 py-1 rounded bg-blue-500 text-white hover:bg-blue-600"
							onclick={selectAllTags}>Select All</button
						>
						<button
							type="button"
							class="px-3 py-1 rounded bg-gray-400 text-white hover:bg-gray-500"
							onclick={clearAllTags}>Deselect All</button
						>
					</div>
				</div>
			{/if}
		</div>
	</div>

	{#if visibleTransactions.length}
		<ul class="space-y-4">
			{#each visibleTransactions as tx (tx.id)}
				<TransactionCard
					transaction={tx}
					allCategories={rawCategories}
					showActions={true}
				/>
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
		<p class="text-gray-500 italic">No transactions match the selected filters.</p>
	{/if}
{/if}
