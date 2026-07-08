<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount, untrack } from 'svelte';
	import { getCategories } from '$lib/api';
	import type { Category, NewCategory } from '$lib/types';
	import { SvelteSet } from 'svelte/reactivity';
	import { resolve } from '$app/paths';

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
	let isAsset = $state(untrack(() => initial.is_asset ?? false));
	let error = $state('');

	// State to hold all categories and mappings so we can derive data synchronously
	let allCategories = $state<Category[]>([]);
	let map = $state(new Map<number, Category>());
	let invalidIds = new SvelteSet<number>();

	onMount(async () => {
		const fetched = await getCategories();
		allCategories = fetched;
		map = new Map(fetched.map((c) => [c.id, c]));

		const newInvalidIds = new SvelteSet<number>();
		if (initial.id) {
			const queue = [initial.id];
			while (queue.length > 0) {
				const currentId = queue.shift()!;
				newInvalidIds.add(currentId);

				const children = fetched.filter((c) => c.parent_id === currentId).map((c) => c.id);
				queue.push(...children);
			}
		}
		invalidIds = newInvalidIds;
	});

	// Reactively compute the normalized name
	let trimmedName = $derived(name.trim().toLowerCase());

	// Reactively filter parents as the user types
	let availableParents = $derived.by(() => {
		if (allCategories.length === 0) return [];

		return allCategories
			.filter((c) => {
				// 1. Prevent circular dependencies (self and descendants)
				if (invalidIds.has(c.id)) return false;

				// 2. Hide parents that already have a child with the current name
				const hasDuplicateChild = allCategories.some(
					(child) =>
						child.parent_id === c.id &&
						child.id !== initial.id && // Ignore self during edits
						child.name.toLowerCase() === trimmedName
				);

				return !hasDuplicateChild;
			})
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

	// Check if there's already a top-level category with this name
	let hasTopLevelDuplicate = $derived(
		allCategories.some(
			(c) =>
				c.parent_id === null && c.id !== initial.id && c.name.toLowerCase() === trimmedName
		)
	);

	// Watcher to reset the selected parent if it becomes invalid while typing
	$effect(() => {
		if (parentId !== 'none') {
			const isCurrentlyValid = availableParents.some((p) => p.id === Number(parentId));
			if (!isCurrentlyValid) {
				parentId = 'none';
			}
		}
	});

	async function submit() {
		error = '';
		const finalName = name.trim();

		if (!finalName) {
			error = 'Name is required.';
			return;
		}

		const selectedParentId = parentId === 'none' ? null : Number(parentId);

		// Final safety net just in case they manage to bypass the UI
		if (selectedParentId === null && hasTopLevelDuplicate) {
			error = 'A top-level category with this name already exists.';
			return;
		}

		const payload: NewCategory = {
			name: finalName,
			parent_id: selectedParentId,
			is_asset: isAsset
		};

		try {
			await onSubmit(payload);
		} catch (e) {
			error = 'Failed to submit category.';
			console.error(e);
		}
	}

	function cancel() {
		goto(resolve('/categories'));
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
			{#if !hasTopLevelDuplicate || parentId === 'none'}
				<option value="none">-- Top Level (No Parent) --</option>
			{/if}

			{#each availableParents as parent ((parent.id, parent.pathName))}
				<option value={parent.id}>{parent.pathName}</option>
			{/each}
		</select>
	</div>

	<div class="flex items-center gap-2 mt-2">
		<input type="checkbox" id="isAsset" bind:checked={isAsset} class="w-4 h-4 rounded" />
		<label for="isAsset" class="font-medium text-gray-700 dark:text-gray-300">
			Track as an Asset Category
		</label>
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
