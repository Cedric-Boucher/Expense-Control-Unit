<script lang="ts">
	import TransactionForm from '$lib/components/TransactionForm.svelte';
	import { createTransaction } from '$lib/api';
	import { goto } from '$app/navigation';
	import type { NewTransaction } from '$lib/types';
	import { resolve } from '$app/paths';

	async function handleCreate(payload: NewTransaction) {
		await createTransaction(payload);
		goto(resolve('/transactions'));
	}

	function cancel() {
		goto(resolve('/transactions'));
	}
</script>

<h1 class="text-2xl font-bold mb-4">New Transaction</h1>
<TransactionForm onSubmit={handleCreate} onCancel={cancel} submitLabel="Create" showCancel={true} />
