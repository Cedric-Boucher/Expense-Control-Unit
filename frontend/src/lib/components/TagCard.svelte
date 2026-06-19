<script lang="ts">
	import type { Tag } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

	let {
		tag,
		showActions = true,
		onEdit = null,
		onDelete = null
	}: {
		tag: Tag;
		showActions?: boolean;
		onEdit?: (() => void) | null;
		onDelete?: (() => void) | null;
	} = $props();

	function handleEdit() {
		if (onEdit) onEdit();
		else
			goto(
				resolve(`/tags/${tag.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`)
			);
	}

	function handleDelete() {
		if (onDelete) onDelete();
		else
			goto(
				resolve(
					`/tags/${tag.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`
				)
			);
	}
</script>

<li class="bg-white dark:bg-gray-800 shadow rounded p-4 flex justify-between items-start gap-4">
	<div>
		<div class="text-2xl font-bold flex items-center gap-3">
			{tag.name}
		</div>
		<div class="text-gray-500 text-sm mt-1">
			Created: {formatTimestampLocalForDisplay(tag.created_at)}
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
