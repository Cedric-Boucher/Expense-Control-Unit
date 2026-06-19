<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount, untrack } from 'svelte';
	import { getTags } from '$lib/api';
	import type { Tag, NewTag } from '$lib/types';
	import { resolve } from '$app/paths';

	let {
		initial = {},
		onSubmit,
		submitLabel = 'Submit',
		showCancel = false
	}: {
		initial?: Partial<Tag>;
		onSubmit: (data: NewTag) => Promise<void>;
		submitLabel?: string;
		showCancel?: boolean;
	} = $props();

	let name = $state(untrack(() => initial.name ?? ''));
	let error = $state('');

	let allTags = $state<Tag[]>([]);

	onMount(async () => {
		allTags = await getTags();
	});

	// Reactively compute the normalized name
	let trimmedName = $derived(name.trim().toLowerCase());

	// Check if there's already a tag with this exact name
	let hasDuplicate = $derived(
		allTags.some((t) => t.id !== initial.id && t.name.toLowerCase() === trimmedName)
	);

	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		const finalName = name.trim();

		if (!finalName) {
			error = 'Name is required.';
			return;
		}

		if (hasDuplicate) {
			error = 'A tag with this name already exists.';
			return;
		}

		const payload: NewTag = {
			name: finalName
		};

		try {
			await onSubmit(payload);
		} catch (e) {
			error = 'Failed to submit tag.';
			console.error(e);
		}
	}

	function cancel() {
		goto(resolve('/tags'));
	}
</script>

<form onsubmit={submit} class="space-y-4 max-w-md">
	<div>
		<label for="name" class="block font-medium">Tag Name</label>
		<input id="name" bind:value={name} class="w-full p-2 border rounded" />
	</div>

	<div class="flex space-x-4 pt-2">
		<button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
			{submitLabel}
		</button>
		{#if showCancel}
			<button
				type="button"
				onclick={cancel}
				class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
			>
				Cancel
			</button>
		{/if}
	</div>

	{#if error}
		<p class="text-red-600 font-medium">{error}</p>
	{/if}
</form>
