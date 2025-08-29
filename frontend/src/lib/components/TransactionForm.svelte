<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { getCategories } from '$lib/api';
    import type { Category, NewTransaction, Transaction } from '$lib/types';
    import { derived, get, writable, type Readable } from 'svelte/store';
    import { formatTimestampLocal } from '$lib/utils';

    export let initial: Partial<Transaction> = {};
    export let onSubmit: (data: NewTransaction) => Promise<void>;
    export let onCancel: () => void;
    export let submitLabel = 'Submit';
    export let showCancel: boolean = false;

    const description = writable(initial.description ?? '');
    const amount = writable(initial.amount?.toString() ?? '');
    const val = parseFloat(get(amount));
    const isExpense = writable(!isNaN(val) ? val < 0 : true);

    const hasInitialTimestamp = !!initial.created_at;
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

    let timestampTouched = false;
    let timer: ReturnType<typeof setInterval> | null = null;

    function formatNumberString(value: string): string {
        if (!value) {
            console.log("format number received: ", value);
            return "";
        }

        // Step 1: keep only digits, and decimal point(s)
        let formatted = value.replace(/[^0-9.]/g, "");
        // Step 2: remove all negative symbols except the first character
        // formatted = formatted.replace(/(?!^)-/g, "");
        // Step 3: remove extra decimal points (keep only the first one)
        formatted = formatted.replace(/(\..*?)\./g, "$1");
        // Step 4: trim leading zeros (but preserve "0" and "-0")
        formatted = formatted.replace(/^(-?)0+(\d)/, "$1$2");

        console.log("Formatted number: ", formatted);

        return formatted;
    }

    function toggleExpense() {
        isExpense.update(v => !v);
    }

    onMount(async () => {
        const result = await getCategories();
        categories.set(result);

        if (initial.category) {
            selectedCategory.set(initial.category);
            inputValue.set(initial.category.name);
        }

        if (!hasInitialTimestamp) {
            const updateTime = () => {
                if (!timestampTouched) {
                    const now = new Date();
                    const tzOffset = now.getTimezoneOffset() * 60000;
                    const localISO = new Date(now.getTime() - tzOffset).toISOString().slice(0, 19);
                    timestamp.set(localISO);
                }
            };
            updateTime();
            timer = setInterval(updateTime, 1000);
        }

        const handleClickOutside = (event: MouseEvent) => {
            if (!categoryContainer.contains(event.target as Node)) {
                showDropdown = false;
            }
        };
        document.addEventListener('click', handleClickOutside, true);
        onDestroy(() => {
            document.removeEventListener('click', handleClickOutside, true);
            if (timer) clearInterval(timer);
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

        if (!$amount || !$selectedCategory) {
            error = 'Category, and amount are required.';
            return;
        }

        const payload: NewTransaction = {
            description: $description,
            amount: Number($amount),
            category_id: $selectedCategory.id,
        };

        if (timestampTouched || hasInitialTimestamp) {
            payload.created_at = toISOStringIfDefined($timestamp || undefined);
        }

        try {
            await onSubmit(payload);
        } catch (e) {
            error = 'Failed to submit transaction.';
            console.error(e);
        }
    }

    function handleTimestampFocusOrInput() {
        timestampTouched = true;
        if (timer) {
            clearInterval(timer);
            timer = null;
        }
    }
</script>

<form on:submit|preventDefault={submit} class="space-y-4 max-w-md">
    <div>
        <label for="amnt" class="block font-medium">Amount</label>
        <input
            id="amnt"
            type="text"
            inputmode="numeric"
            bind:value={$amount}
            on:input={
                (e) => {
                    const input = e.target as HTMLInputElement;
                    amount.set(formatNumberString(input.value.toString()));
                    input.value = get(amount);
                }
            }
            class="w-full p-2 border rounded {$isExpense ? 'text-red-800 dark:text-red-200' : 'text-green-800 dark:text-green-200'}"
        />
    </div>

    <div>
        <span class="font-medium {$isExpense ? 'text-red-600' : 'text-gray-500 dark:text-gray-300'}">Expense</span>
        <!-- svelte-ignore a11y_consider_explicit_label -->
        <button
            type="button"
            role="switch"
            aria-checked={$isExpense}
            on:click={toggleExpense}
            class="relative inline-flex h-6 w-12 items-center rounded-full transition-colors focus:outline-none
                {$isExpense ? 'bg-red-600' : 'bg-green-600'}"
        >
            <span
                class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform
                    {$isExpense ? 'translate-x-1' : 'translate-x-7'}"
            ></span>
        </button>
        <span class="font-medium {$isExpense ? 'text-gray-500 dark:text-gray-300' : 'text-green-600'}">Income</span>
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
        <label for="desc" class="block font-medium">Description</label>
        <input id="desc" bind:value={$description} class="w-full p-2 border rounded" />
    </div>

    <div>
        <label for="time" class="block font-medium">Timestamp</label>
        <input
            id="time"
            type="datetime-local"
            bind:value={$timestamp}
            step="1"
            class="w-full p-2 border rounded"
            on:focus={handleTimestampFocusOrInput}
            on:input={handleTimestampFocusOrInput}
        />
    </div>

    <div class="flex space-x-4">
        <button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
            {submitLabel}
        </button>
        {#if showCancel}
            <button
                type="button"
                on:click={onCancel}
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
