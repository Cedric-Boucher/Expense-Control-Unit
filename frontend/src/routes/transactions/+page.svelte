<script lang="ts">
	import { getTransactions } from '$lib/api';
	import type { Transaction } from '$lib/types';
    import { onMount } from 'svelte';

    let transactions: Transaction[] = [];

    onMount(async () => {
        transactions = await getTransactions()
    });
</script>

<h1>Transactions</h1>

{#if transactions.length}
    <ul>
        {#each transactions as tx}
            <li>{tx.description} - ${tx.amount}</li>
        {/each}
    </ul>
{:else}
    <p>No transactions found.</p>
{/if}
