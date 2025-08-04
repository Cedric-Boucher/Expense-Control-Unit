<script lang="ts">
    import { page } from '$app/state';
    import { getTransaction, deleteTransaction } from '$lib/api';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import type { Transaction } from '$lib/types';
	import TransactionCard from '$lib/components/TransactionCard.svelte';

    let transaction: Transaction | null = null;
    let error = '';
    let loading = true;

    const id = page.params.id;

    onMount(async () => {
        if (!id) {
            goto('/transactions');
        } else {
            try {
                transaction = await getTransaction(id);
            } catch (e) {
                error = 'Failed to load transaction.';
                console.error(e);
            } finally {
                loading = false;
            }
        }
    });

    async function confirmDelete() {
        try {
            if (id) {
                await deleteTransaction(id);
            }
            goto('/transactions');
        } catch (e) {
            error = 'Failed to delete transaction.';
            console.error(e);
        }
    }

    function cancel() {
        goto('/transactions');
    }
</script>

{#if loading}
    <p>Loading...</p>
{:else if error}
    <p class="text-red-600">{error}</p>
{:else if transaction}
    <h1 class="text-2xl font-bold mb-4">Delete Transaction</h1>
    <p class="mb-2">Are you sure you want to delete the following transaction?</p>

    <TransactionCard {transaction} showActions={false} />

    <div class="flex space-x-4">
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
    <p class="text-gray-500 italic">Transaction not found.</p>
{/if}
