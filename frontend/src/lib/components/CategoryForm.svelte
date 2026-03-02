<script lang="ts">
    import { writable, get } from 'svelte/store';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { getCategories } from '$lib/api';
    import type { Category, NewCategory } from '$lib/types';

    export let initial: Partial<Category> = {};
    export let onSubmit: (data: NewCategory) => Promise<void>;
    export let submitLabel = 'Submit';
    export let showCancel = false;

    const name = writable(initial.name ?? '');
    const parentId = writable<number | string>(initial.parent_id ?? 'none');
    let error = '';
    
    let availableParents: { id: number, pathName: string }[] = [];

    onMount(async () => {
        const fetched = await getCategories();
        const map = new Map(fetched.map(c => [c.id, c]));

        const validParents = initial.id ? fetched.filter(c => c.id !== initial.id) : fetched;

        availableParents = validParents.map(c => {
            let path = c.name;
            let curr = c;
            while (curr.parent_id && map.has(curr.parent_id)) {
                curr = map.get(curr.parent_id)!;
                path = curr.name + ' / ' + path;
            }
            return { id: c.id, pathName: path };
        }).sort((a, b) => a.pathName.localeCompare(b.pathName));
    });

    async function submit() {
        error = '';
        const $name = get(name).trim();
        const $parentId = get(parentId);

        if (!$name) {
            error = 'Name is required.';
            return;
        }

        const payload: NewCategory = { 
            name: $name,
            parent_id: $parentId === 'none' ? null : Number($parentId)
        };

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
        <input id="name" bind:value={$name} class="w-full p-2 border rounded" />
    </div>

    <div>
        <label for="parent" class="block font-medium">Parent Category</label>
        <select id="parent" bind:value={$parentId} class="w-full p-2 border rounded bg-white dark:bg-gray-800">
            <option value="none">-- Top Level (No Parent) --</option>
            {#each availableParents as parent}
                <option value={parent.id}>{parent.pathName}</option>
            {/each}
        </select>
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
