<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { useAppData, useUpdateTag } from '$lib/queries';
	import TagForm from '$lib/components/TagForm.svelte';
	import type { NewTag } from '$lib/types';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';

	const appData = useAppData();
	const updateTagMutation = useUpdateTag();

	let id = $derived(page.params.id);
	let numericId = $derived(Number(id));
	let redirectTo = $derived((page.url.searchParams.get('redirectTo') ?? '/tags') as Pathname);

	let tag = $derived(appData.data?.tags?.find((t) => t.id === numericId) ?? null);

	$effect(() => {
		if (!id) {
			if (redirectTo) goto(resolve(redirectTo));
		}
	});

	async function handleUpdate(data: NewTag) {
		if (id) {
			await updateTagMutation.mutateAsync({ id, data });
		}
		await goto(resolve(redirectTo));
	}
</script>

{#if appData.isPending}
	<p>Loading...</p>
{:else if tag}
	<h1 class="text-2xl font-bold mb-4">Edit Tag</h1>
	<TagForm initial={tag} onSubmit={handleUpdate} submitLabel="Save Changes" showCancel={true} />
{:else}
	<p>Tag not found.</p>
{/if}
