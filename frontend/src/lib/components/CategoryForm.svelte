<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount, untrack } from 'svelte';
	import { getCategories } from '$lib/api';
	import type { Category, NewCategory } from '$lib/types';
	import { SvelteSet } from 'svelte/reactivity';

	let {
		initial = {},
		onSubmit,
		submitLabel = 'Submit',
		showCancel = false
	}: {
		initial?: Partial<Category>;
		onSubmit: (data: NewCategory) => Promise<void>;
		submitLabel?: string;
		showCancel?: boolean;
	} = $props();

	let name = $state(untrack(() => initial.name ?? ''));
	let parentId = $state<number | string>(untrack(() => initial.parent_id ?? 'none'));
	let error = $state('');

	let availableParents = $state<{ id: number; pathName: string }[]>([]);

	onMount(async () => {
		const fetched = await getCategories();
		const map = new Map(fetched.map((c) => [c.id, c]));

		const invalidIds = new SvelteSet<number>();
		if (initial.id) {
			const queue = [initial.id];
			while (queue.length > 0) {
				const currentId = queue.shift()!;
				invalidIds.add(currentId);

				const children = fetched.filter((c) => c.parent_id === currentId).map((c) => c.id);
				queue.push(...children);
			}
		}

		const validParents = fetched.filter((c) => !invalidIds.has(c.id));

		availableParents = validParents
			.map((c) => {
				let path = c.name;
				let curr = c;
				while (curr.parent_id && map.has(curr.parent_id)) {
					curr = map.get(curr.parent_id)!;
					path = curr.name + ' / ' + path;
				}
				return { id: c.id, pathName: path };
			})
			.sort((a, b) => a.pathName.localeCompare(b.pathName));
	});

	async function submit() {
		error = '';
		const trimmedName = name.trim();

		if (!trimmedName) {
			error = 'Name is required.';
			return;
		}

		const payload: NewCategory = {
			name: trimmedName,
			parent_id: parentId === 'none' ? null : Number(parentId)
		};

		try {
			await onSubmit(payload);
		} catch (e) {
			error = 'Failed to submit category.';
			console.error(e);
		}
	}

	function cancel() {
		goto('/categories');
	}
</script>

<form onsubmit={submit} class="space-y-4 max-w-md">
	<div>
		<label for="name" class="block font-medium">Category Name</label>
		<input id="name" bind:value={name} class="w-full p-2 border rounded" />
	</div>

	<div>
		<label for="parent" class="block font-medium">Parent Category</label>
		<select
			id="parent"
			bind:value={parentId}
			class="w-full p-2 border rounded bg-white dark:bg-gray-800"
		>
			<option value="none">-- Top Level (No Parent) --</option>
			{#each availableParents as parent ((parent.id, parent.pathName))}
				<option value={parent.id}>{parent.pathName}</option>
			{/each}
		</select>
	</div>

	<div class="flex space-x-4">
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
		<p class="text-red-600">{error}</p>
	{/if}
</form>
