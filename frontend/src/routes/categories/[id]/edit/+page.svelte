<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/state';
    import { goto } from '$app/navigation';
    import { getCategory, updateCategory } from '$lib/api';
    import CategoryForm from '$lib/components/CategoryForm.svelte';
    import type { Category, NewCategory } from '$lib/types';

    let category: Category | null = null;
    $: id = page.params.id;

    onMount(async () => {
        if (id) {
            category = await getCategory(id);
        } else {
            goto('/categories');
        }
    });

    async function handleUpdate(data: NewCategory) {
        if (id) {
            await updateCategory(id, data);
        }
        goto('/categories');
    }
</script>

{#if category}
    <h1 class="text-2xl font-bold mb-4">Edit Category</h1>
    <CategoryForm initial={category} onSubmit={handleUpdate} submitLabel="Save Changes" showCancel={true} />
{:else}
    <p>Loading...</p>
{/if}
