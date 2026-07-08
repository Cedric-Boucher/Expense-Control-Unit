<script lang="ts">
	import { page } from '$app/state';
	import { getTag, deleteTag } from '$lib/api';
	import { goto } from '$app/navigation';
	import type { Tag } from '$lib/types';
	import TagCard from '$lib/components/TagCard.svelte';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';

	let tag = $state<Tag | null>(null);
	let error = $state('');
	let loading = $state(true);

	let id = $derived(page.params.id);
	let redirectTo = $derived((page.url.searchParams.get('redirectTo') ?? '/tags') as Pathname);

	$effect(() => {
		if (id) {
			loadData(id);
		} else {
			if (redirectTo) goto(resolve(redirectTo));
		}
	});

	async function loadData(currentId: string) {
		loading = true;
		error = '';

		try {
			tag = await getTag(currentId);
		} catch (e) {
			error = 'Failed to load tag data.';
			console.error(e);
		} finally {
			loading = false;
		}
	}

	async function confirmDelete() {
		try {
			if (id) {
				await deleteTag(id);
			}
			goto(resolve(redirectTo));
		} catch (e) {
			error = 'Failed to delete tag.';
			console.error(e);
		}
	}

	function cancel() {
		goto(resolve(redirectTo));
	}
</script>

{#if loading}
	<p>Loading...</p>
{:else if error}
	<p class="text-red-600">{error}</p>
{:else if tag}
	<h1 class="text-2xl font-bold mb-4">Delete Tag</h1>

	<p class="mb-2">Are you sure you want to delete the following tag?</p>
	<TagCard {tag} showActions={false} />

	<p class="mt-4 text-sm text-gray-700 dark:text-gray-300">
		Deleting this tag will automatically remove it from any associated transactions. The
		transactions themselves will not be deleted.
	</p>

	<div class="flex space-x-4 mt-4">
		<button
			onclick={confirmDelete}
			class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
		>
			Yes, Delete
		</button>
		<button
			onclick={cancel}
			class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
		>
			Cancel
		</button>
	</div>
{:else}
	<p class="text-gray-500 italic">Tag not found.</p>
{/if}
