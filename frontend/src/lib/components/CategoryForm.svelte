<script lang="ts">
    import { writable, get } from 'svelte/store';
    import { goto } from '$app/navigation';
    import type { Category, NewCategory } from '$lib/types';

    export let initial: Partial<Category> = {};
    export let onSubmit: (data: NewCategory) => Promise<void>;
    export let submitLabel = 'Submit';
    export let showCancel = false;

    const name = writable(initial.name ?? '');
    let error = '';

    async function submit() {
        error = '';
        const $name = get(name).trim();

        if (!$name) {
            error = 'Name is required.';
            return;
        }

        const payload: NewCategory = { name: $name };

        try {
            await onSubmit(payload);
        } catch (e) {
            error = 'Failed to submit category.';
            console.error(e);
        }
    }

    function cancel() {
        goto('/categories');
    }
</script>

<form on:submit|preventDefault={submit} class="space-y-4 max-w-md">
    <div>
        <label for="name" class="block font-medium">Category Name</label>
        <input
            id="name"
            bind:value={$name}
            class="w-full p-2 border rounded"
        />
    </div>

    <div class="flex space-x-4">
        <button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
            {submitLabel}
        </button>
        {#if showCancel}
            <button
                type="button"
                on:click={cancel}
                class="bg-gray-300 dark:bg-gray-700 text-black dark:text-white px-4 py-2 rounded hover:bg-gray-400 dark:hover:bg-gray-600"
            >
                Cancel
            </button>
        {/if}
    </div>

    {#if error}
        <p class="text-red-600">{error}</p>
    {/if}
</form>
