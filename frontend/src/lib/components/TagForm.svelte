<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import { useAppData } from '$lib/queries';
	import type { Tag, NewTag } from '$lib/types';
	import { formatTimestampLocal } from '$lib/utils';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';

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

	const appData = useAppData();

	let name = $state(untrack(() => initial.name ?? ''));
	let closingDate = $state(
		untrack(() => (initial.closing_date ? formatTimestampLocal(initial.closing_date) : ''))
	);
	let error = $state('');

	let allTags = $derived(appData.data?.tags ?? []);

	let trimmedName = $derived(name.trim().toLowerCase());

	let hasDuplicate = $derived(
		allTags.some((t) => t.id !== initial.id && t.name.toLowerCase() === trimmedName)
	);

	const toISOStringIfDefined = (str: string | undefined | null) =>
		str ? new Date(str).toISOString() : null;

	async function submit() {
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
			name: finalName,
			closing_date: toISOStringIfDefined(closingDate)
		};

		try {
			await onSubmit(payload);
		} catch (e) {
			error = 'Failed to submit tag.';
			console.error(e);
		}
	}

	async function cancel() {
		await goto(resolve('/tags'));
	}
</script>

<form onsubmit={(e) => e.preventDefault()} class="space-y-4 max-w-md">
	<div>
		<label for="name" class="block font-medium">Tag Name</label>
		<input
			id="name"
			bind:value={name}
			class="w-full p-2 border rounded"
			placeholder="e.g. 2006-honda-civic"
		/>
	</div>

	<div>
		<label for="closingDate" class="block font-medium">Closing Date (Optional)</label>
		<p class="text-xs text-gray-500 mb-1">
			If this tag represents an asset you sold or disposed of, set the date it was closed.
			Leave blank if it is currently active.
		</p>
		<div class="flex items-center gap-2">
			<input
				id="closingDate"
				type="datetime-local"
				bind:value={closingDate}
				step="1"
				class="w-full p-2 border rounded flex-1"
			/>
			{#if closingDate}
				<button
					type="button"
					class="px-3 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 text-sm"
					onclick={() => (closingDate = '')}
				>
					Clear
				</button>
			{/if}
		</div>
	</div>

	<div class="flex space-x-4 pt-2">
		<AsyncButton
			type="submit"
			action={submit}
			class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
			disabled={appData.isPending}
		>
			{submitLabel}
		</AsyncButton>

		{#if showCancel}
			<AsyncButton
				type="button"
				action={cancel}
				class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
			>
				Cancel
			</AsyncButton>
		{/if}
	</div>

	{#if error}
		<p class="text-red-600 font-medium">{error}</p>
	{/if}
</form>
