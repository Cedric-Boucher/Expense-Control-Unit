<script lang="ts">
    import { page } from '$app/state';
    import { getCategory, deleteCategory, getCategoryTransactions } from '$lib/api';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import type { Category, Transaction } from '$lib/types';
    import CategoryCard from '$lib/components/CategoryCard.svelte';
    import TransactionCard from '$lib/components/TransactionCard.svelte';

    let category: Category | null = null;
    let transactions: Transaction[] = [];
    let error = '';
    let loading = true;

    const id = page.params.id;

    onMount(async () => {
        if (!id) {
            goto('/categories');
            return;
        }

        try {
            category = await getCategory(id);
            transactions = await getCategoryTransactions(id);
        } catch (e) {
            error = 'Failed to load category or transactions.';
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

    {#if transactions.length > 0}
        <p class="mt-4 text-red-600">
            This category is currently used by the following transaction{transactions.length > 1 ? 's' : ''}:
        </p>
        <ul class="space-y-2 mt-2">
            {#each transactions as tx}
                <TransactionCard transaction={tx} showActions={true} />
            {/each}
        </ul>

        <p class="mt-4 text-sm text-gray-700 dark:text-gray-300">
            Please update or delete {transactions.length > 1 ? 'these transactions' : 'this transaction'} before deleting this category.
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
