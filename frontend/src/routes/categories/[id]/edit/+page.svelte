<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { getCategory, updateCategory } from '$lib/api';
	import CategoryForm from '$lib/components/CategoryForm.svelte';
	import type { Category, NewCategory } from '$lib/types';

	let category = $state<Category | null>(null);

	let id = $derived(page.params.id);
	let redirectTo = $derived(page.url.searchParams.get('redirectTo') ?? '/categories');

	$effect(() => {
		if (id) {
			loadData(id);
		} else {
			if (redirectTo) goto(redirectTo);
		}
	});

	async function loadData(currentId: string) {
		category = null;
		try {
			category = await getCategory(currentId);
		} catch (e) {
			console.error('Failed to load category:', e);
		}
	}

	async function handleUpdate(data: NewCategory) {
		if (id) {
			await updateCategory(id, data);
		}
		goto(redirectTo);
	}
</script>

{#if category}
	<h1 class="text-2xl font-bold mb-4">Edit Category</h1>
	<CategoryForm
		initial={category}
		onSubmit={handleUpdate}
		submitLabel="Save Changes"
		showCancel={true}
	/>
{:else}
	<p>Loading...</p>
{/if}
