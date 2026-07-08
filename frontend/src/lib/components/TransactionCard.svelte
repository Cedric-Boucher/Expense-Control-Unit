<script lang="ts">
	import type { Transaction, Category } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

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
				resolve(
					`/transactions/${transaction.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`
				)
			);
	}

	function handleDelete() {
		if (onDelete) onDelete();
		else
			goto(
				resolve(
					`/transactions/${transaction.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`
				)
			);
	}
</script>

<li class="bg-white dark:bg-gray-800 shadow rounded p-4 flex justify-between items-start gap-4">
	<div>
		<div class="text-2xl font-bold flex items-center gap-3">
			<div>
				{#if path.parentPath}
					<span class="text-gray-400 font-normal text-lg">{path.parentPath}</span>
				{/if}
				{path.name}
			</div>
			{#if transaction.category.is_asset}
				<span
					class="text-xs bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 px-2 py-1 rounded-full font-semibold uppercase tracking-wide"
				>
					Asset
				</span>
			{/if}
		</div>
		<div class="font-medium mt-1">{transaction.description}</div>
		<div
			class={transaction.amount > 0
				? 'text-green-600 font-bold text-xl'
				: 'text-red-600 font-bold text-xl'}
		>
			${transaction.amount.toFixed(2)}
		</div>
		<div class="text-gray-500 text-sm mt-1">
			{formatTimestampLocalForDisplay(transaction.created_at)}
		</div>

		{#if transaction.tags && transaction.tags.length > 0}
			<div class="flex flex-wrap gap-2 mt-3">
				{#each transaction.tags as tag (tag)}
					<span
						class="text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-600 px-2 py-0.5 rounded-full font-medium"
					>
						{tag.name}
					</span>
				{/each}
			</div>
		{/if}
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
