<script lang="ts">
    import { goto } from '$app/navigation';
    import type { Category } from '$lib/types';
    import CategoryCard from '$lib/components/CategoryCard.svelte';

    export let data: { categories: Category[] };

    const BATCH_SIZE = 50; 

    const ESTIMATED_ITEM_HEIGHT = 80; 

    let limit = BATCH_SIZE;

    $: {
        data.categories;
        limit = BATCH_SIZE;
        if (typeof window !== 'undefined') window.scrollTo(0, 0);
    }

    $: if (limit < data.categories.length) {
        setTimeout(() => {
            limit += BATCH_SIZE;
        }, 0);
    }

    $: visibleCategories = data.categories.slice(0, limit);

    $: remainingCount = Math.max(0, data.categories.length - limit);
    $: phantomHeight = remainingCount * ESTIMATED_ITEM_HEIGHT;
</script>

<h1 class="text-2xl font-bold mb-4">Categories</h1>

<div class="mb-6">
    <button
        class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
        on:click={() => goto('/categories/new')}
    >
        + New Category
    </button>
</div>

{#if visibleCategories.length}
    <ul class="space-y-4">
        {#each visibleCategories as category (category.id)}
            <CategoryCard {category} showActions={true} />
        {/each}

        <div style="height: {phantomHeight}px; width: 100%"></div>
    </ul>
{:else}
    <p class="text-gray-500 italic">No categories found.</p>
{/if}
