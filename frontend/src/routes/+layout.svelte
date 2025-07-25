<script lang="ts">
    import { goto } from '$app/navigation';
    import '../app.css';
</script>

<nav class="bg-white shadow sticky top-0 z-10">
    <div class="max-w-4xl mx-auto px-4 py-3 flex gap-4">
        <button
            class="px-3 py-2 rounded hover:bg-blue-100 text-blue-700 font-medium"
            on:click={() => goto('/')}>
            Home
        </button>
        <button
            class="px-3 py-2 rounded hover:bg-blue-100 text-blue-700 font-medium"
            on:click={() => goto('/transactions')}>
            Transactions
        </button>
        <button
            class="px-3 py-2 rounded hover:bg-blue-100 text-blue-700 font-medium"
            on:click={() => goto('/new_transaction')}>
            New Transaction
        </button>
    </div>
</nav>

<main class="max-w-4xl mx-auto px-4 py-6">
    <slot />
</main>
