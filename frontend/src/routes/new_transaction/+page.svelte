<script lang="ts">
	import { goto } from '$app/navigation';
	import { createTransaction } from '$lib/api';
	import type { NewTransaction } from '$lib/types';

	let description = '';
	let amount: number | '' = '';
	let timestamp: string = ''; // ISO 8601 string

	let error = '';
	let success = false;

	async function submit() {
		error = '';
		success = false;

		if (!description || !amount) {
			error = 'Description and amount are required.';
			return;
		}

		const payload: NewTransaction = {
			description,
			amount: Number(amount),
			created_at: timestamp || undefined,
		};

		try {
			await createTransaction(payload);
			success = true;
			goto('/transactions');
		} catch (e) {
			error = 'Failed to submit transaction.';
			console.error(e);
		}
	}
</script>

<h1 class="text-2xl font-bold mb-4">New Transaction</h1>

<form on:submit|preventDefault={submit} class="space-y-4 max-w-md">
	<div>
		<label for="desc" class="block font-medium">Description</label>
		<input id="desc" bind:value={description} class="w-full p-2 border rounded" />
	</div>

	<div>
		<label for="amt" class="block font-medium">Amount</label>
		<input id="amt" type="number" step="0.01" bind:value={amount} class="w-full p-2 border rounded" />
	</div>

	<div>
		<label for="time" class="block font-medium">Timestamp (optional)</label>
		<input id="time" type="datetime-local" bind:value={timestamp} class="w-full p-2 border rounded" />
	</div>

	<button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
		Submit
	</button>

	{#if error}
		<p class="text-red-600">{error}</p>
	{/if}
</form>
