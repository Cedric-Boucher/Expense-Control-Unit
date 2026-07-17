<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import TransactionCard from '$lib/components/TransactionCard.svelte';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';
	import AsyncButton from '$lib/components/AsyncButton.svelte';
	import { useDeleteTransaction, useAppData } from '$lib/queries';

	let error = $state('');

	let id = $derived(page.params.id);
	let redirectTo = $derived(
		(page.url.searchParams.get('redirectTo') ?? '/transactions') as Pathname
	);

	const deleteTx = useDeleteTransaction();
	const appData = useAppData();

	let transaction = $derived(appData.data?.transactions.find((t) => String(t.id) === id) || null);

	$effect(() => {
		if (!id && redirectTo) {
			goto(resolve(redirectTo));
		}
	});

	async function confirmDelete() {
		error = '';
		try {
			if (id) {
				await deleteTx.mutateAsync(id);
			}
			await goto(resolve(redirectTo));
		} catch (e) {
			error = 'Failed to delete transaction.';
			console.error(e);
		}
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
	<h1 class="text-2xl font-bold mb-4">Delete Transaction</h1>
	<p class="mb-2">Are you sure you want to delete the following transaction?</p>

	<TransactionCard {transaction} showActions={false} />

	{#if error}
		<p class="text-red-600 mt-2">{error}</p>
	{/if}

	<div class="flex space-x-4 mt-4">
		<AsyncButton
			action={confirmDelete}
			class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
		>
			Yes, Delete
		</AsyncButton>
		<AsyncButton
			action={cancel}
			class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
		>
			Cancel
		</AsyncButton>
	</div>
{:else}
	<p class="text-gray-500 italic">Transaction not found.</p>
{/if}
