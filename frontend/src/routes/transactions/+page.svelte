<script lang="ts">
    import { onMount } from 'svelte';
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import type { Transaction } from '$lib/types';
    import TransactionCard from '$lib/components/TransactionCard.svelte';
    export let data: { transactions: Transaction[] };

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });
</script>

<h1 class="text-2xl font-bold mb-4">Transactions</h1>

<div class="mb-6">
    <button
        class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
        on:click={() => goto('/transactions/new')}
    >
        + New Transaction
    </button>
</div>

{#if data.transactions.length}
    <ul class="space-y-4">
        {#each data.transactions as tx}
            <TransactionCard transaction={tx} showActions={true} />
        {/each}
    </ul>
{:else}
    <p class="text-gray-500 italic">No transactions found.</p>
{/if}
