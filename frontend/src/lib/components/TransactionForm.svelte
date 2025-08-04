<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { getCategories } from '$lib/api';
    import type { Category, NewTransaction, Transaction } from '$lib/types';
    import { derived, get, writable, type Readable } from 'svelte/store';
    import { formatTimestampLocal } from '$lib/utils';
	import { goto } from '$app/navigation';

    export let initial: Partial<Transaction> = {};
    export let onSubmit: (data: NewTransaction) => Promise<void>;
    export let submitLabel = 'Submit';
    export let showCancel: boolean = false;

    const description = writable(initial.description ?? '');
    const amount = writable(initial.amount ?? '');
    const timestamp = writable(initial.created_at ? formatTimestampLocal(initial.created_at) : '');

    const categories = writable<Category[]>([]);
    const inputValue = writable('');
    const selectedCategory = writable<Category | null>(null);
    let showDropdown = false;
    let error = '';
    let categoryContainer: HTMLDivElement;

    const filtered: Readable<Category[]> = derived([categories, inputValue], ([$categories, $inputValue]) =>
        $categories.filter(cat => cat.name.toLowerCase().startsWith($inputValue.toLowerCase()))
    );

    onMount(async () => {
        const result = await getCategories();
        categories.set(result);

        // Pre-fill selected category
        if (initial.category) {
            selectedCategory.set(initial.category);
            inputValue.set(initial.category.name);
        }

        const handleClickOutside = (event: MouseEvent) => {
            if (!categoryContainer.contains(event.target as Node)) {
                showDropdown = false;
            }
        };
        document.addEventListener('click', handleClickOutside, true);
        onDestroy(() => {
            document.removeEventListener('click', handleClickOutside, true);
        });
    });

    const toISOStringIfDefined = (str: string | undefined) =>
        str ? new Date(str).toISOString() : undefined;

    function handleSelect(category: Category) {
        selectedCategory.set(category);
        inputValue.set(category.name);
        showDropdown = false;
    }

    async function submit() {
        error = '';

        const $description = get(description);
        const $amount = get(amount);
        const $selectedCategory = get(selectedCategory);
        const $timestamp = get(timestamp);

        if (!$description || !$amount || !$selectedCategory) {
            error = 'Description, category, and amount are required.';
            return;
        }

        const payload: NewTransaction = {
            description: $description,
            amount: Number($amount),
            created_at: toISOStringIfDefined($timestamp || undefined),
            category_id: $selectedCategory.id,
        };

        try {
            await onSubmit(payload);
        } catch (e) {
            error = 'Failed to submit transaction.';
            console.error(e);
        }
    }

    function cancel() {
        goto('/transactions');
    }
</script>

<form on:submit|preventDefault={submit} class="space-y-4 max-w-md">
    <div>
        <label for="desc" class="block font-medium">Description</label>
        <input id="desc" bind:value={$description} class="w-full p-2 border rounded" />
    </div>

    <div bind:this={categoryContainer}>
        <label for="cat" class="block font-medium">Category</label>
        <input
            id="cat"
            type="text"
            bind:value={$inputValue}
            on:input={() => (showDropdown = true)}
            on:focus={() => (showDropdown = true)}
            placeholder="Select category..."
            class="w-full p-2 border rounded"
        />
        {#if showDropdown}
            <ul class="absolute z-10 bg-white dark:bg-gray-800 border w-fit mt-1 max-h-60 overflow-auto shadow rounded">
                {#each $filtered as category (category.id)}
                    <li class="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-950 cursor-pointer">
                        <button on:click={() => handleSelect(category)}>{category.name}</button>
                    </li>
                {/each}
                {#if $filtered.length === 0}
                    <li class="px-3 py-2 text-gray-500 dark:text-gray-300">No matches found</li>
                {/if}
            </ul>
        {/if}
    </div>

    <div>
        <label for="amnt" class="block font-medium">Amount</label>
        <input id="amnt" type="number" step="0.01" bind:value={$amount} class="w-full p-2 border rounded" />
    </div>

    <div>
        <label for="time" class="block font-medium">Timestamp (optional)</label>
        <input id="time" type="datetime-local" bind:value={$timestamp} step="1" class="w-full p-2 border rounded" />
    </div>

    <div class="flex space-x-4">
        <button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
            {submitLabel}
        </button>
        {#if showCancel}
            <button
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
