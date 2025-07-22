<script>
  import { onMount } from 'svelte';
  let transactions = [];

  onMount(async () => {
    const res = await fetch('http://localhost:8000/api/transactions');
    transactions = await res.json();
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
