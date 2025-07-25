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

<h1>New Transaction</h1>

<form on:submit|preventDefault={submit} class="form">
	<div>
		<label for="desc">Description</label>
		<input id="desc" bind:value={description} placeholder="e.g., Coffee" />
	</div>

	<div>
		<label for="amt">Amount (use negative for expenses)</label>
		<input id="amt" type="number" step="0.01" bind:value={amount} />
	</div>

	<div>
		<label for="time">Custom Timestamp (optional)</label>
		<input
			id="time"
			type="datetime-local"
			bind:value={timestamp}
		/>
	</div>

	<button type="submit">Submit</button>

	{#if error}
		<p style="color: red">{error}</p>
	{/if}
</form>
