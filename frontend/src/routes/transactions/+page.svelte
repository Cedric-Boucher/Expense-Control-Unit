<script lang="ts">
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { getTransactions } from '$lib/api';
    import type { Transaction } from '$lib/types';
    import { onMount } from 'svelte';
    import { formatTimestampLocal } from '$lib/utils';

    let transactions: Transaction[] = [];

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });

    onMount(async () => {
        transactions = await getTransactions()
    });
</script>

<h1 class="text-2xl font-bold mb-4">Transactions</h1>

{#if transactions.length}
    <ul class="space-y-4">
        {#each transactions as tx}
            <li class="bg-white dark:bg-gray-800 shadow rounded p-4">
                <div class="text-lg font-medium">{tx.description}</div>
                <div class="text-green-600 font-bold">${tx.amount}</div>
                <div class="text-gray-500 text-sm">{formatTimestampLocal(tx.created_at)}</div>
            </li>
        {/each}
    </ul>
{:else}
    <p class="text-gray-500 italic">No transactions found.</p>
{/if}
