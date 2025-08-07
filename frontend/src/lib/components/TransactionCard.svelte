<script lang="ts">
    import type { Transaction } from '$lib/types';
    import { formatTimestampLocalForDisplay } from '$lib/utils';
    import { goto } from '$app/navigation';
	import { page } from '$app/state';

    export let transaction: Transaction;
    export let showActions: boolean = true;

    // Optional overrides
    export let onEdit: (() => void) | null = null;
    export let onDelete: (() => void) | null = null;

    function handleEdit() {
        if (onEdit) {
            onEdit();
        } else {
            goto(`/transactions/${transaction.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`);
        }
    }

    function handleDelete() {
        if (onDelete) {
            onDelete();
        } else {
            goto(`/transactions/${transaction.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`);
        }
    }
</script>

<li class="bg-white dark:bg-gray-800 shadow rounded p-4 flex justify-between items-start gap-4">
    <!-- Left: transaction details -->
    <div>
        <div class="text-2xl font-bold">{transaction.category.name}</div>
        <div class="font-medium">{transaction.description}</div>
        <div class={transaction.amount > 0 ? 'text-green-600 font-bold text-xl' : 'text-red-600 font-bold text-xl'}>
            ${transaction.amount}
        </div>
        <div class="text-gray-500 text-sm">{formatTimestampLocalForDisplay(transaction.created_at)}</div>
    </div>

    <!-- Right: actions -->
    {#if showActions}
        <div class="flex flex-col gap-2">
            <button
                type="button"
                class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
                on:click={handleEdit}
            >
                Edit
            </button>
            <button
                type="button"
                class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
                on:click={handleDelete}
            >
                Delete
            </button>
        </div>
    {/if}
</li>
