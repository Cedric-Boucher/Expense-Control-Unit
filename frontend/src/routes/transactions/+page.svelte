<script lang="ts">
	import { getTransactions } from '$lib/api';
	import type { Transaction } from '$lib/types';
    import { onMount } from 'svelte';
    import { formatTimestampLocal } from '$lib/utils';

    let transactions: Transaction[] = [];

    onMount(async () => {
        transactions = await getTransactions()
    });
</script>

<h1>Transactions</h1>

{#if transactions.length}
    <ul>
        {#each transactions as tx}
            <li>
				<strong>{tx.description}</strong> — ${tx.amount}  
				<br />
				<small>{formatTimestampLocal(tx.created_at)}</small>
			</li>
        {/each}
    </ul>
{:else}
    <p>No transactions found.</p>
{/if}
