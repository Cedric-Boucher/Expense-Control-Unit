<script>
  import { goto } from '$app/navigation';
</script>

<nav>
  <button on:click={() => goto('/')}>Home</button>
  <button on:click={() => goto('/transactions')}>Transactions</button>
</nav>

<slot /> <!-- This renders the current page -->
