<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { useCreateTag } from '$lib/queries';
	import TagForm from '$lib/components/TagForm.svelte';
	import type { NewTag } from '$lib/types';

	const createTagMutation = useCreateTag();

	async function handleSubmit(data: NewTag) {
		await createTagMutation.mutateAsync(data);
		await goto(resolve('/tags'));
	}
</script>

<h1 class="text-2xl font-bold mb-4">New Tag</h1>

<TagForm onSubmit={handleSubmit} submitLabel="Create" showCancel={true} />
