<script lang="ts">
	import { page } from '$app/state';
	import { getTransaction, deleteTransaction } from '$lib/api';
	import { goto } from '$app/navigation';
	import type { Transaction } from '$lib/types';
	import TransactionCard from '$lib/components/TransactionCard.svelte';

	let transaction = $state<Transaction | null>(null);
	let error = $state('');
	let loading = $state(true);

	let id = $derived(page.params.id);
	let redirectTo = $derived(page.url.searchParams.get('redirectTo') ?? '/transactions');

	$effect(() => {
		if (id) {
			loadData(id);
		} else {
			if (redirectTo) goto(redirectTo);
		}
	});

	async function loadData(currentId: string) {
		loading = true;
		error = '';

		try {
			transaction = await getTransaction(currentId);
		} catch (e) {
			error = 'Failed to load transaction.';
			console.error(e);
		} finally {
			loading = false;
		}
	}

	async function confirmDelete() {
		try {
			if (id) {
				await deleteTransaction(id);
			}
			goto(redirectTo);
		} catch (e) {
			error = 'Failed to delete transaction.';
			console.error(e);
		}
	}

	function cancel() {
		goto(redirectTo);
	}
</script>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="text-red-600">{error}</p>
{:else if transaction}
	<h1 class="text-2xl font-bold mb-4">Delete Transaction</h1>
	<p class="mb-2">Are you sure you want to delete the following transaction?</p>

	<TransactionCard {transaction} showActions={false} />

	<div class="flex space-x-4 mt-4">
		<button
			onclick={confirmDelete}
			class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
		>
			Yes, Delete
		</button>
		<button
			onclick={cancel}
			class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
		>
			Cancel
		</button>
	</div>
{:else}
	<p class="text-gray-500 italic">Transaction not found.</p>
{/if}
