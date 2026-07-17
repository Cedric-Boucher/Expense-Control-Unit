<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import type { Category, NewCategory } from '$lib/types';
	import { SvelteSet } from 'svelte/reactivity';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';
	import { useAppData } from '$lib/queries';

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

	const appData = useAppData();

	let name = $state(untrack(() => initial.name ?? ''));
	let parentId = $state<number | string>(untrack(() => initial.parent_id ?? 'none'));
	let isAsset = $state(untrack(() => initial.is_asset ?? false));
	let error = $state('');

	// Derive all categories and their mapping directly from the query cache
	let allCategories = $derived(appData.data?.categories ?? []);
	let map = $derived(new Map(allCategories.map((c) => [c.id, c])));

	// Use a single SvelteSet instance and mutate it with set methods
	const invalidIds = new SvelteSet<number>();

	// Compute the invalid descendants whenever the categories list updates
	$effect(() => {
		invalidIds.clear();

		if (initial.id && allCategories.length > 0) {
			const queue = [initial.id];
			while (queue.length > 0) {
				const currentId = queue.shift()!;
				invalidIds.add(currentId);

				const children = allCategories
					.filter((c) => c.parent_id === currentId)
					.map((c) => c.id);
				queue.push(...children);
			}
		}
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
		if (appData.isPending) return; // Guard: Do nothing until data has successfully loaded

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

	async function cancel() {
		await goto(resolve('/categories'));
	}
</script>

<form onsubmit={(e) => e.preventDefault()} class="space-y-4 max-w-md">
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
			disabled={appData.isPending}
		>
			{#if !hasTopLevelDuplicate || parentId === 'none'}
				<option value="none">-- Top Level (No Parent) --</option>
			{/if}

			{#if appData.isPending}
				<option value="none">Loading categories...</option>
			{:else}
				{#each availableParents as parent ((parent.id, parent.pathName))}
					<option value={parent.id}>{parent.pathName}</option>
				{/each}
			{/if}
		</select>
	</div>

	<div class="flex items-center gap-2 mt-2">
		<input type="checkbox" id="isAsset" bind:checked={isAsset} class="w-4 h-4 rounded" />
		<label for="isAsset" class="font-medium text-gray-700 dark:text-gray-300">
			Track as an Asset Category
		</label>
	</div>

	<div class="flex space-x-4 pt-2">
		<AsyncButton
			type="submit"
			action={submit}
			class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
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
