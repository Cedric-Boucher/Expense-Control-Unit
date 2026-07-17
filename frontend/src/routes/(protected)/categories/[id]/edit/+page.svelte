<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { useAppData, useUpdateCategory } from '$lib/queries';
	import CategoryForm from '$lib/components/CategoryForm.svelte';
	import type { NewCategory } from '$lib/types';
	import { resolve } from '$app/paths';
	import type { Pathname } from '$app/types';

	const appData = useAppData();
	const updateCategory = useUpdateCategory();

	let id = $derived(page.params.id);
	let numericId = $derived(Number(id));
	let redirectTo = $derived(
		(page.url.searchParams.get('redirectTo') ?? '/categories') as Pathname
	);

	let category = $derived(appData.data?.categories.find((c) => c.id === numericId) ?? null);

	$effect(() => {
		if (!id) {
			if (redirectTo) goto(resolve(redirectTo));
		}
	});

	async function handleUpdate(data: NewCategory) {
		if (id) {
			await updateCategory.mutateAsync({ id, data });
		}
		await goto(resolve(redirectTo));
	}
</script>

{#if appData.isPending}
	<p>Loading...</p>
{:else if appData.isError}
	<p class="text-red-600">Failed to load category data.</p>
{:else if category}
	<h1 class="text-2xl font-bold mb-4">Edit Category</h1>
	<CategoryForm
		initial={category}
		onSubmit={handleUpdate}
		submitLabel="Save Changes"
		showCancel={true}
	/>
{:else}
	<p class="text-gray-500 italic">Category not found.</p>
{/if}
