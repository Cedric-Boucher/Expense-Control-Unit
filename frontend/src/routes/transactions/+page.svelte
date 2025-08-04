<script lang="ts">
    import { getTransactions } from '$lib/api';
    import { onMount } from 'svelte';
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import type { Transaction } from '$lib/types';
    import TransactionCard from '$lib/components/TransactionCard.svelte';

    let transactions: Transaction[] = [];

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });

    onMount(async () => {
        transactions = await getTransactions();
    });
</script>

<h1 class="text-2xl font-bold mb-4">Transactions</h1>

{#if transactions.length}
    <ul class="space-y-4">
        {#each transactions as tx}
            <TransactionCard transaction={tx} showActions={true} />
        {/each}
    </ul>
{:else}
    <p class="text-gray-500 italic">No transactions found.</p>
{/if}
