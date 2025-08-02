<script lang="ts">
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { createTransaction, getCategories } from '$lib/api';
    import type { Category, NewTransaction } from '$lib/types';
    import { onDestroy, onMount } from 'svelte';
    import { derived, writable, type Readable } from 'svelte/store';

    let categoryContainer: HTMLDivElement;

    onMount(() => {
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

    let description = '';
    const inputValue = writable('');
    let amount: number | '' = '';
    let timestamp: string = ''; // ISO 8601 string
    let showDropdown = false;

    let error = '';

    export let selectedCategory: Category | '' = '';
    export let onSelect: (category: Category) => void;

    const categories = writable<Category[]>([]);
    const filtered: Readable<Category[]> = derived([categories, inputValue], ([$categories, $inputValue]) =>
        $categories.filter((cat: Category) =>
        cat.name.toLowerCase().startsWith($inputValue.toLowerCase())
        )
    );

    onMount(async () => {
        const result = await getCategories();
        categories.set(result);
    });

    function handleSelect(category: Category) {
        selectedCategory = category;
        $inputValue = category.name;
        showDropdown = false;
        onSelect(category);
    }

    onMount(() => {
        if (!$auth.isLoggedIn) {
            goto('/login');
        }
    });

    async function submit() {
        error = '';

        if (!description || !amount || !selectedCategory) {
            error = 'Description, category and amount are required.';
            return;
        }

        const payload: NewTransaction = {
            description,
            category_id: Number(selectedCategory.id),
            amount: Number(amount),
            created_at: timestamp || undefined,
        };

        try {
            await createTransaction(payload);
            goto('/transactions');
        } catch (e) {
            error = 'Failed to submit transaction.';
            console.error(e);
        }
    }
</script>

<h1 class="text-2xl font-bold mb-4">New Transaction</h1>

<form on:submit|preventDefault={submit} class="space-y-4 max-w-md">
    <div>
        <label for="desc" class="block font-medium">Description</label>
        <input id="desc" bind:value={description} class="w-full p-2 border rounded" />
    </div>

    <div bind:this={categoryContainer}>
        <label for="category" class="block font-medium">Category</label>
        <input
            id="category"
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
                    <button on:click={() => handleSelect(category)}>
                        {category.name}
                    </button>
                </li>
            {/each}
            {#if $filtered.length === 0}
                <li class="px-3 py-2 text-gray-500 dark:text-gray-300">No matches found</li>
            {/if}
            </ul>
        {/if}
    </div>

    <div>
        <label for="amt" class="block font-medium">Amount</label>
        <input id="amt" type="number" step="0.01" bind:value={amount} class="w-full p-2 border rounded" />
    </div>

    <div>
        <label for="time" class="block font-medium">Timestamp (optional)</label>
        <input id="time" type="datetime-local" bind:value={timestamp} class="w-full p-2 border rounded" />
    </div>

    <button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
        Submit
    </button>

    {#if error}
        <p class="text-red-600">{error}</p>
    {/if}
</form>

<style>
    ul::-webkit-scrollbar {
        width: 8px;
    }
    ul::-webkit-scrollbar-thumb {
        background-color: #ccc;
        border-radius: 4px;
    }
</style>
