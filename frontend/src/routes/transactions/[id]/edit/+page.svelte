<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/state';
    import { getTransaction, updateTransaction } from '$lib/api';
    import { goto } from '$app/navigation';
    import TransactionForm from '$lib/components/TransactionForm.svelte';
    import type { NewTransaction, Transaction } from '$lib/types';

    let transaction: Transaction | null = null;
    const id = page.params.id;
    const redirectTo = page.url.searchParams.get('redirectTo') ?? '/transactions';

    onMount(async () => {
        if (id) {
            transaction = await getTransaction(id);
        } else {
            goto(redirectTo);
        }
    });

    async function handleUpdate(data: NewTransaction) {
        if (id) {
            await updateTransaction(id, data);
        }
        goto(redirectTo);
    }
</script>

{#if transaction}
    <h1 class="text-2xl font-bold mb-4">Edit Transaction</h1>
    <TransactionForm initial={transaction} onSubmit={handleUpdate} submitLabel="Save Changes" showCancel={true} />
{:else}
    <p>Loading...</p>
{/if}
