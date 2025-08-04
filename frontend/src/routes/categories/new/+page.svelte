<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { auth } from '$lib/stores/auth';
    import { createCategory } from '$lib/api';
    import CategoryForm from '$lib/components/CategoryForm.svelte';
    import type { NewCategory } from '$lib/types';

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });

    async function handleSubmit(data: NewCategory) {
        await createCategory(data);
        goto('/categories');
    }
</script>

<h1 class="text-2xl font-bold mb-4">New Category</h1>

<CategoryForm onSubmit={handleSubmit} submitLabel="Create" showCancel={true} />
