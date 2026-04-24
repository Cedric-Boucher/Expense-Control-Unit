<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import type { Category } from '$lib/types';
	import CategoryCard from '$lib/components/CategoryCard.svelte';

	let { data }: { data: { categories: Category[] } } = $props();

	const BATCH_SIZE = 50;
	const ESTIMATED_ITEM_HEIGHT = 80;

	let limit = $state(BATCH_SIZE);

	function loadMore() {
		if (limit < data.categories.length) {
			limit += BATCH_SIZE;
			setTimeout(loadMore, 0);
		}
	}

	$effect(() => {
		if (data.categories) {
			untrack(() => {
				limit = BATCH_SIZE;
				if (typeof window !== 'undefined') window.scrollTo(0, 0);
				setTimeout(loadMore, 0);
			});
		}
	});

	let visibleCategories = $derived(data.categories.slice(0, limit));
	let remainingCount = $derived(Math.max(0, data.categories.length - limit));
	let phantomHeight = $derived(remainingCount * ESTIMATED_ITEM_HEIGHT);
</script>

<h1 class="text-2xl font-bold mb-4">Categories</h1>

<div class="mb-6">
	<button
		class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
		onclick={() => goto('/categories/new')}
	>
		+ New Category
	</button>
</div>

{#if visibleCategories.length}
	<ul class="space-y-4">
		{#each visibleCategories as category (category.id)}
			<CategoryCard {category} allCategories={data.categories} showActions={true} />
		{/each}

		<div style="height: {phantomHeight}px; width: 100%"></div>
	</ul>
{:else}
	<p class="text-gray-500 italic">No categories found.</p>
{/if}
