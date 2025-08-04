<script lang="ts">
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { createCategory } from '$lib/api';
    import type { NewCategory } from '$lib/types';
    import { onMount } from 'svelte';

    let name = '';

    let error = '';

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });

    async function submit() {
        error = '';

        if (!name) {
            error = 'Name is required.';
            return;
        }

        const payload: NewCategory = {
            name
        };

        try {
            await createCategory(payload);
            goto('/transactions');
        } catch (e) {
            error = 'Failed to submit category.';
            console.error(e);
        }
    }

    function cancel() {
        goto('/categories');
    }
</script>

<h1 class="text-2xl font-bold mb-4">New Category</h1>

<form on:submit|preventDefault={submit} class="space-y-4 max-w-md">
    <div>
        <label for="name" class="block font-medium">Name</label>
        <input id="name" bind:value={name} class="w-full p-2 border rounded" />
    </div>

    <button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
        Submit
    </button>
    <button
        type="button"
        on:click={cancel}
        class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
    >
        Cancel
    </button>
    {#if error}
        <p class="text-red-600">{error}</p>
    {/if}
</form>
