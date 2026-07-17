<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import CategoryCard from '$lib/components/CategoryCard.svelte';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';
	import { useAppData } from '$lib/queries';

	const appData = useAppData();

	let categories = $derived(appData.data?.categories ?? []);

	const BATCH_SIZE = 50;
	const ESTIMATED_ITEM_HEIGHT = 80;

	let limit = $state(BATCH_SIZE);

	function loadMore() {
		if (limit < categories.length) {
			limit += BATCH_SIZE;
			setTimeout(loadMore, 0);
		}
	}

	$effect(() => {
		if (categories.length > 0) {
			untrack(() => {
				limit = BATCH_SIZE;
				if (typeof window !== 'undefined') window.scrollTo(0, 0);
				setTimeout(loadMore, 0);
			});
		}
	});

	let visibleCategories = $derived(categories.slice(0, limit));
	let remainingCount = $derived(Math.max(0, categories.length - limit));
	let phantomHeight = $derived(remainingCount * ESTIMATED_ITEM_HEIGHT);
</script>

<h1 class="text-2xl font-bold mb-4">Categories</h1>

<div class="mb-6">
	<AsyncButton
		class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
		action={async () => await goto(resolve('/categories/new'))}
	>
		+ New Category
	</AsyncButton>
</div>

{#if appData.isPending}
	<p>Loading categories...</p>
{:else if appData.isError}
	<p class="text-red-600">Failed to load categories.</p>
{:else if visibleCategories.length}
	<ul class="space-y-4">
		{#each visibleCategories as category (category.id)}
			<CategoryCard {category} allCategories={categories} showActions={true} />
		{/each}

		<div style="height: {phantomHeight}px; width: 100%"></div>
	</ul>
{:else}
	<p class="text-gray-500 italic">No categories found.</p>
{/if}
