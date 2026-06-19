<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { getTag, updateTag } from '$lib/api';
	import TagForm from '$lib/components/TagForm.svelte';
	import type { Tag, NewTag } from '$lib/types';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';

	let tag = $state<Tag | null>(null);

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
		tag = null;
		try {
			tag = await getTag(currentId);
		} catch (e) {
			console.error('Failed to load tag:', e);
		}
	}

	async function handleUpdate(data: NewTag) {
		if (id) {
			await updateTag(id, data);
		}
		goto(resolve(redirectTo));
	}
</script>

{#if tag}
	<h1 class="text-2xl font-bold mb-4">Edit Tag</h1>
	<TagForm initial={tag} onSubmit={handleUpdate} submitLabel="Save Changes" showCancel={true} />
{:else}
	<p>Loading...</p>
{/if}
