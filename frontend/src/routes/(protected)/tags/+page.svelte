<script lang="ts">
	import { goto } from '$app/navigation';
	import { untrack } from 'svelte';
	import TagCard from '$lib/components/TagCard.svelte';
	import { resolve } from '$app/paths';
	import AsyncButton from '$lib/components/AsyncButton.svelte';
	import { useAppData } from '$lib/queries';

	const appData = useAppData();

	// Derive the tags from the query cache, defaulting to an empty array
	let tags = $derived(appData.data?.tags ?? []);

	const BATCH_SIZE = 50;
	const ESTIMATED_ITEM_HEIGHT = 80;

	let limit = $state(BATCH_SIZE);

	function loadMore() {
		if (limit < tags.length) {
			limit += BATCH_SIZE;
			setTimeout(loadMore, 0);
		}
	}

	$effect(() => {
		// Trigger the batching load mechanism once tags are populated
		if (tags.length > 0) {
			untrack(() => {
				limit = BATCH_SIZE;
				if (typeof window !== 'undefined') window.scrollTo(0, 0);
				setTimeout(loadMore, 0);
			});
		}
	});

	let visibleTags = $derived(tags.slice(0, limit));
	let remainingCount = $derived(Math.max(0, tags.length - limit));
	let phantomHeight = $derived(remainingCount * ESTIMATED_ITEM_HEIGHT);
</script>

<h1 class="text-2xl font-bold mb-4">Tags</h1>

<div class="mb-6">
	<AsyncButton
		class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
		action={async () => await goto(resolve('/tags/new'))}
	>
		+ New Tag
	</AsyncButton>
</div>

{#if appData.isPending}
	<p>Loading tags...</p>
{:else if appData.isError}
	<p class="text-red-600">Failed to load tags.</p>
{:else if visibleTags.length}
	<ul class="space-y-4">
		{#each visibleTags as tag (tag.id)}
			<TagCard {tag} showActions={true} />
		{/each}

		<div style="height: {phantomHeight}px; width: 100%"></div>
	</ul>
{:else}
	<p class="text-gray-500 italic">No tags found.</p>
{/if}
