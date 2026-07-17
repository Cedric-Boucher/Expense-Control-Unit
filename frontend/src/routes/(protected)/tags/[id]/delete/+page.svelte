<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { useAppData, useDeleteTag } from '$lib/queries';
	import TagCard from '$lib/components/TagCard.svelte';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';
	import AsyncButton from '$lib/components/AsyncButton.svelte';

	const appData = useAppData();
	const deleteTagMutation = useDeleteTag();

	let actionError = $state('');

	let id = $derived(page.params.id);
	let numericId = $derived(Number(id));
	let redirectTo = $derived((page.url.searchParams.get('redirectTo') ?? '/tags') as Pathname);

	let tag = $derived(appData.data?.tags?.find((t) => t.id === numericId));

	$effect(() => {
		if (!id) {
			if (redirectTo) goto(resolve(redirectTo));
		}
	});

	async function confirmDelete() {
		try {
			if (id) {
				await deleteTagMutation.mutateAsync(id);
			}
			await goto(resolve(redirectTo));
		} catch (e) {
			actionError = 'Failed to delete tag.';
			console.error(e);
		}
	}

	async function cancel() {
		await goto(resolve(redirectTo));
	}
</script>

{#if appData.isPending}
	<p>Loading...</p>
{:else if appData.isError}
	<p class="text-red-600">Failed to load data.</p>
{:else if tag}
	<h1 class="text-2xl font-bold mb-4">Delete Tag</h1>

	<p class="mb-2">Are you sure you want to delete the following tag?</p>
	<TagCard {tag} showActions={false} />

	<p class="mt-4 text-sm text-gray-700 dark:text-gray-300">
		Deleting this tag will automatically remove it from any associated transactions. The
		transactions themselves will not be deleted.
	</p>

	<div class="flex space-x-4 mt-4">
		<AsyncButton
			action={confirmDelete}
			class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
		>
			Yes, Delete
		</AsyncButton>
		<AsyncButton
			action={cancel}
			class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
		>
			Cancel
		</AsyncButton>
	</div>
	{#if actionError}
		<p class="text-red-600 mt-4">{actionError}</p>
	{/if}
{:else}
	<p class="text-gray-500 italic">Tag not found.</p>
{/if}
