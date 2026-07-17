<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { useCreateCategory } from '$lib/queries';
	import CategoryForm from '$lib/components/CategoryForm.svelte';
	import type { NewCategory } from '$lib/types';

	const createCategory = useCreateCategory();

	async function handleSubmit(data: NewCategory) {
		await createCategory.mutateAsync(data);
		await goto(resolve('/categories'));
	}
</script>

<h1 class="text-2xl font-bold mb-4">New Category</h1>

<CategoryForm onSubmit={handleSubmit} submitLabel="Create" showCancel={true} />
