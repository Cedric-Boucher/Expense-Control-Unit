<script lang="ts">
    import { page } from '$app/state';
    import { getCategory, deleteCategory } from '$lib/api';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import type { Category } from '$lib/types';
    import CategoryCard from '$lib/components/CategoryCard.svelte';

    let category: Category | null = null;
    let error = '';
    let loading = true;

    const id = page.params.id;

    onMount(async () => {
        if (!id) {
            goto('/categories');
        } else {
            try {
                category = await getCategory(id);
            } catch (e) {
                error = 'Failed to load category.';
                console.error(e);
            } finally {
                loading = false;
            }
        }
    });

    async function confirmDelete() {
        try {
            if (id) {
                await deleteCategory(id);
            }
            goto('/categories');
        } catch (e) {
            error = 'Failed to delete category.';
            console.error(e);
        }
    }

    function cancel() {
        goto('/categories');
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
{:else}
    <p class="text-gray-500 italic">Category not found.</p>
{/if}

<!-- TODO: delete page should tell user they cannot delete if there are transactions associated, and list the transactions (with the action buttons)-->
