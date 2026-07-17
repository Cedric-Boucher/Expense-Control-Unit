<script lang="ts">
	import TransactionForm from '$lib/components/TransactionForm.svelte';
	import { goto } from '$app/navigation';
	import type { NewTransaction } from '$lib/types';
	import { resolve } from '$app/paths';
	import { useCreateTransaction } from '$lib/queries';

	const createTx = useCreateTransaction();

	async function handleCreate(payload: NewTransaction) {
		await createTx.mutateAsync(payload);
		await goto(resolve('/transactions'));
	}

	async function cancel() {
		await goto(resolve('/transactions'));
	}
</script>

<h1 class="text-2xl font-bold mb-4">New Transaction</h1>
<TransactionForm onSubmit={handleCreate} onCancel={cancel} submitLabel="Create" showCancel={true} />
