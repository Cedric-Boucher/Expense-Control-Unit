<script lang="ts">
	import type { Category } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';

	let {
		category,
		allCategories = [],
		showActions = true,
		onEdit = null,
		onDelete = null
	}: {
		category: Category;
		allCategories?: Category[];
		showActions?: boolean;
		onEdit?: (() => void) | null;
		onDelete?: (() => void) | null;
	} = $props();

	let path = $derived(computePathParts(category, allCategories));

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

	async function handleEdit() {
		if (onEdit) onEdit();
		else
			await goto(
				resolve(
					`/categories/${category.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`
				)
			);
	}

	async function handleDelete() {
		if (onDelete) onDelete();
		else
			await goto(
				resolve(
					`/categories/${category.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`
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
			{#if category.is_asset}
				<span
					class="text-xs bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200 px-2 py-1 rounded-full font-semibold uppercase tracking-wide"
				>
					Asset
				</span>
			{/if}
		</div>
		<div class="text-gray-500 text-sm mt-1">
			Created: {formatTimestampLocalForDisplay(category.created_at)}
		</div>
	</div>

	{#if showActions}
		<div class="flex flex-col gap-2">
			<AsyncButton
				class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
				action={handleEdit}>Edit</AsyncButton
			>
			<AsyncButton
				class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
				action={handleDelete}>Delete</AsyncButton
			>
		</div>
	{/if}
</li>
