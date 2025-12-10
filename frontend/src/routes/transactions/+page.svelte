<script lang="ts">
    import { goto } from '$app/navigation';
    import { writable, derived } from 'svelte/store';
    import { onMount, onDestroy } from 'svelte';
    import type { Transaction, Category } from '$lib/types';
    import TransactionCard from '$lib/components/TransactionCard.svelte';

    export let data: { transactions: Transaction[] };

    const categories: Category[] = Array.from(
        new Map(data.transactions.map(tx => [tx.category.id, tx.category])).values()
    ).sort((a, b) => a.name.localeCompare(b.name));

    const selectedCategoryIds = writable<Set<number>>(new Set(categories.map(c => c.id)));
    const filteredTransactions = derived(selectedCategoryIds, $selected =>
        data.transactions.filter(tx => $selected.has(tx.category.id))
    );

    const BATCH_SIZE = 100;
    const ESTIMATED_ITEM_HEIGHT = 80; 

    let limit = BATCH_SIZE;

    $: {
        $filteredTransactions;
        limit = BATCH_SIZE;
        if (typeof window !== 'undefined') window.scrollTo(0, 0);
    }

    $: if (limit < $filteredTransactions.length) {
        setTimeout(() => {
            limit += BATCH_SIZE;
        }, 0);
    }

    $: visibleTransactions = $filteredTransactions.slice(0, limit);

    $: remainingCount = Math.max(0, $filteredTransactions.length - limit);
    $: phantomHeight = remainingCount * ESTIMATED_ITEM_HEIGHT;

    const isFilterOpen = writable(false);
    let filterContainer: HTMLDivElement;
    const searchQuery = writable('');

    const visibleCategories = derived(searchQuery, $searchQuery =>
        categories.filter(cat =>
            cat.name.toLowerCase().includes($searchQuery.toLowerCase())
        )
    );

    function toggleCategory(id: number) {
        selectedCategoryIds.update(set => {
            const newSet = new Set(set);
            newSet.has(id) ? newSet.delete(id) : newSet.add(id);
            return newSet;
        });
    }

    function selectAll() {
        selectedCategoryIds.set(new Set(categories.map(c => c.id)));
    }

    function clearAll() {
        selectedCategoryIds.set(new Set());
    }

    onMount(() => {
        function handleClickOutside(event: MouseEvent) {
            if (filterContainer && !filterContainer.contains(event.target as Node)) {
                isFilterOpen.set(false);
            }
        }
        document.addEventListener('click', handleClickOutside);
        onDestroy(() => document.removeEventListener('click', handleClickOutside));
    });
</script>

<h1 class="text-2xl font-bold mb-4">Transactions</h1>

<div class="mb-6 flex items-center gap-4">
    <button
        class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
        on:click={() => goto('/transactions/new')}
    >
        + New Transaction
    </button>

    <div class="relative" bind:this={filterContainer}>
        <button
            type="button"
            class="bg-gray-200 dark:bg-gray-700 px-4 py-2 rounded hover:bg-gray-300 dark:hover:bg-gray-600 flex items-center gap-2"
            on:click={() => isFilterOpen.update(v => !v)}
        >
            <span class="font-medium">Filter Categories</span>
            <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
            </svg>
        </button>

        {#if $isFilterOpen}
            <div class="absolute left-0 mt-2 w-64 bg-gray-100 dark:bg-gray-800 p-4 rounded shadow-lg z-10">
                <input
                    type="text"
                    placeholder="Search categories..."
                    class="w-full p-2 mb-3 border rounded dark:bg-gray-700 dark:text-white"
                    bind:value={$searchQuery}
                />

                <div class="flex flex-col gap-2 max-h-64 overflow-auto">
                    {#if $visibleCategories.length > 0}
                        {#each $visibleCategories as cat}
                            <label class="flex items-center space-x-2">
                                <input
                                    type="checkbox"
                                    checked={$selectedCategoryIds.has(cat.id)}
                                    on:change={() => toggleCategory(cat.id)}
                                />
                                <span>{cat.name}</span>
                            </label>
                        {/each}
                    {:else}
                        <p class="text-sm text-gray-500 italic">No categories found</p>
                    {/if}
                </div>

                <div class="flex justify-between mt-3">
                    <button
                        type="button"
                        class="px-3 py-1 rounded bg-blue-500 text-white hover:bg-blue-600"
                        on:click={selectAll}
                    >
                        Select All
                    </button>
                    <button
                        type="button"
                        class="px-3 py-1 rounded bg-gray-400 text-white hover:bg-gray-500"
                        on:click={clearAll}
                    >
                        Deselect All
                    </button>
                </div>
            </div>
        {/if}
    </div>
</div>

{#if visibleTransactions.length}
    <ul class="space-y-4">
        {#each visibleTransactions as tx (tx.id)}
            <TransactionCard transaction={tx} showActions={true} />
        {/each}

        <div style="height: {phantomHeight}px; width: 100%"></div>
    </ul>

    {#if limit < $filteredTransactions.length}
        <p class="text-center text-xs text-gray-400 mt-2">Loading rest of data...</p>
    {:else}
        <p class="text-center text-xs text-gray-400 mt-2 mb-8">
            Showing all {$filteredTransactions.length} transactions
        </p>
    {/if}
{:else}
    <p class="text-gray-500 italic">No transactions match the selected categories.</p>
{/if}
