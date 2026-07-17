<script lang="ts">
	import type { Tag } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';

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

	async function handleEdit() {
		if (onEdit) onEdit();
		else
			await goto(
				resolve(`/tags/${tag.id}/edit?redirectTo=${encodeURIComponent(page.url.pathname)}`)
			);
	}

	async function handleDelete() {
		if (onDelete) onDelete();
		else
			await goto(
				resolve(
					`/tags/${tag.id}/delete?redirectTo=${encodeURIComponent(page.url.pathname)}`
				)
			);
	}
</script>

<li
	class="bg-white dark:bg-gray-800 shadow rounded p-4 flex justify-between items-start gap-4 border-l-4 {tag.closing_date
		? 'border-gray-400'
		: 'border-green-500'}"
>
	<div>
		<div class="text-2xl font-bold flex items-center gap-3">
			{tag.name}
			{#if tag.closing_date}
				<span
					class="text-xs bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300 px-2 py-1 rounded-full font-semibold uppercase tracking-wide"
				>
					Closed
				</span>
			{:else}
				<span
					class="text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 px-2 py-1 rounded-full font-semibold uppercase tracking-wide"
				>
					Active
				</span>
			{/if}
		</div>
		<div class="text-gray-500 text-sm mt-2 flex flex-col gap-1">
			<span>Created: {formatTimestampLocalForDisplay(tag.created_at)}</span>
			{#if tag.closing_date}
				<span class="text-gray-600 dark:text-gray-400 font-medium">
					Closed on: {formatTimestampLocalForDisplay(tag.closing_date)}
				</span>
			{/if}
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
