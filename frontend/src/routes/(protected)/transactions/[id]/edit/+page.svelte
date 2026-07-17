<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import TransactionForm from '$lib/components/TransactionForm.svelte';
	import type { NewTransaction } from '$lib/types';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';
	import { useUpdateTransaction, useAppData } from '$lib/queries';

	let id = $derived(page.params.id);
	let redirectTo = $derived(
		(page.url.searchParams.get('redirectTo') ?? '/transactions') as Pathname
	);

	const updateTx = useUpdateTransaction();
	const appData = useAppData();

	let transaction = $derived(appData.data?.transactions.find((t) => String(t.id) === id) || null);

	$effect(() => {
		if (!id && redirectTo) {
			goto(resolve(redirectTo));
		}
	});

	async function handleUpdate(data: NewTransaction) {
		if (id) {
			await updateTx.mutateAsync({ id, data });
		}
		await goto(resolve(redirectTo));
	}

	async function cancel() {
		await goto(resolve(redirectTo));
	}
</script>

{#if appData.isPending}
	<p>Loading...</p>
{:else if appData.isError}
	<p class="text-red-600">Failed to load transaction data.</p>
{:else if transaction}
	<h1 class="text-2xl font-bold mb-4">Edit Transaction</h1>
	<TransactionForm
		initial={transaction}
		onSubmit={handleUpdate}
		onCancel={cancel}
		submitLabel="Save Changes"
		showCancel={true}
	/>
{:else}
	<p class="text-gray-500 italic">Transaction not found.</p>
{/if}
