<script lang="ts">
	import { page } from '$app/state';
	import { getTransaction, updateTransaction } from '$lib/api';
	import { goto } from '$app/navigation';
	import TransactionForm from '$lib/components/TransactionForm.svelte';
	import type { NewTransaction, Transaction } from '$lib/types';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';

	let transaction = $state<Transaction | null>(null);

	let id = $derived(page.params.id);
	let redirectTo = $derived(
		(page.url.searchParams.get('redirectTo') ?? '/transactions') as Pathname
	);

	$effect(() => {
		if (id) {
			loadData(id);
		} else {
			if (redirectTo) goto(resolve(redirectTo));
		}
	});

	async function loadData(currentId: string) {
		transaction = null;
		try {
			transaction = await getTransaction(currentId);
		} catch (e) {
			console.error('Failed to load transaction:', e);
		}
	}

	async function handleUpdate(data: NewTransaction) {
		if (id) {
			await updateTransaction(id, data);
		}
		goto(resolve(redirectTo));
	}

	function cancel() {
		goto(resolve(redirectTo));
	}
</script>

{#if transaction}
	<h1 class="text-2xl font-bold mb-4">Edit Transaction</h1>
	<TransactionForm
		initial={transaction}
		onSubmit={handleUpdate}
		onCancel={cancel}
		submitLabel="Save Changes"
		showCancel={true}
	/>
{:else}
	<p>Loading...</p>
{/if}
