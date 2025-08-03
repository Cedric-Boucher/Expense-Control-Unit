<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/state';
    import { goto } from '$app/navigation';
    import { getTransaction, updateTransaction } from '$lib/api';
    import type { Transaction } from '$lib/types';

    let transaction: Transaction | null = null;

    // Get transaction ID from route
    $: id = page.params.id;

    onMount(async () => {
        if (!id) {
            goto('/transactions'); // something is wrong with the URL and id cannot be found
        }
        else {
            transaction = await getTransaction(id);
        }
    });

    async function submit() {
        if (id && transaction) {
            await updateTransaction(id, transaction);
        }
        goto('/transactions');
    }
</script>

{#if transaction}
    <h1 class="text-2xl font-bold mb-4">Edit Transaction</h1>

    <form on:submit|preventDefault={submit} class="space-y-4">
        <div>
            <label for="desc">Description</label>
            <input id="desc" bind:value={transaction.description} class="input" />
        </div>
        <div>
            <label for="amnt">Amount</label>
            <input id="amnt" type="number" bind:value={transaction.amount} class="input" />
        </div>
        <div>
            <label for="time">Timestamp</label>
            <input id="time" type="datetime-local" bind:value={transaction.created_at} class="input" />
        </div>
        <!-- Add category editing if you support it -->
        <button type="submit" class="btn btn-primary">Save</button>
    </form>
{:else}
    <p>Loading...</p>
{/if}
