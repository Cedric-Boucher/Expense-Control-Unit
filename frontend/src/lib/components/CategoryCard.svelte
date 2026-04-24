<script lang="ts">
	import type { Category } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

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

	function handleEdit() {
		if (onEdit) onEdit();
		else
			goto(
				`/categories/${category.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`
			);
	}

	function handleDelete() {
		if (onDelete) onDelete();
		else
			goto(
				`/categories/${category.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`
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
		<div class="text-gray-500 text-sm">
			Created: {formatTimestampLocalForDisplay(category.created_at)}
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
