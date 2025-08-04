<script lang="ts">
    import { getCategories } from '$lib/api';
    import { onMount } from 'svelte';
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import type { Category } from '$lib/types';
    import CategoryCard from '$lib/components/CategoryCard.svelte';

    let categories: Category[] = [];

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });

    onMount(async () => {
        categories = await getCategories();
    });
</script>

<h1 class="text-2xl font-bold mb-4">Categories</h1>

{#if categories.length}
    <ul class="space-y-4">
        {#each categories as category}
            <CategoryCard {category} showActions={true} />
        {/each}
    </ul>
{:else}
    <p class="text-gray-500 italic">No categories found.</p>
{/if}
