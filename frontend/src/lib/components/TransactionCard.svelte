<script lang="ts">
	import type { Transaction, Category } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let {
		transaction,
		allCategories = [],
		showActions = true,
		onEdit = null,
		onDelete = null
	}: {
		transaction: Transaction;
		allCategories?: Category[];
		showActions?: boolean;
		onEdit?: (() => void) | null;
		onDelete?: (() => void) | null;
	} = $props();

	let path = $derived(computePathParts(transaction.category, allCategories));

	function computePathParts(targetCat: Category, cats: Category[]) {
		if (!cats.length) return { parentPath: '', name: targetCat.name };

		const map = new Map(cats.map((c) => [c.id, c]));
		let current = targetCat;
		const parents = [];

		while (current.parent_id && map.has(current.parent_id)) {
			current = map.get(current.parent_id)!;
			parents.unshift(current.name);
		}

		return {
			parentPath: parents.length > 0 ? parents.join(' / ') + ' / ' : '',
			name: targetCat.name
		};
	}

	function handleEdit() {
		if (onEdit) onEdit();
		else
			goto(
				`/transactions/${transaction.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`
			);
	}

	function handleDelete() {
		if (onDelete) onDelete();
		else
			goto(
				`/transactions/${transaction.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`
			);
	}
</script>

<li class="bg-white dark:bg-gray-800 shadow rounded p-4 flex justify-between items-start gap-4">
	<div>
		<div class="text-2xl font-bold">
			{#if path.parentPath}
				<span class="text-gray-400 font-normal text-lg">{path.parentPath}</span>
			{/if}
			{path.name}
		</div>
		<div class="font-medium">{transaction.description}</div>
		<div
			class={transaction.amount > 0
				? 'text-green-600 font-bold text-xl'
				: 'text-red-600 font-bold text-xl'}
		>
			${transaction.amount.toFixed(2)}
		</div>
		<div class="text-gray-500 text-sm">
			{formatTimestampLocalForDisplay(transaction.created_at)}
		</div>
	</div>

	{#if showActions}
		<div class="flex flex-col gap-2">
			<button
				type="button"
				class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
				onclick={handleEdit}>Edit</button
			>
			<button
				type="button"
				class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
				onclick={handleDelete}>Delete</button
			>
		</div>
	{/if}
</li>
