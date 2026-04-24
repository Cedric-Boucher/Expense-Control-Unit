<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { getCategories } from '$lib/api';
	import type { Category, NewTransaction, Transaction } from '$lib/types';
	import { formatTimestampLocal } from '$lib/utils';

	let {
		initial = {},
		onSubmit,
		onCancel,
		submitLabel = 'Submit',
		showCancel = false
	}: {
		initial?: Partial<Transaction>;
		onSubmit: (data: NewTransaction) => Promise<void>;
		onCancel: () => void;
		submitLabel?: string;
		showCancel?: boolean;
	} = $props();

	let description = $state(untrack(() => initial.description ?? ''));
	let amount = $state(untrack(() => formatNumberString(initial.amount?.toString() ?? '')));

	const val = parseFloat(untrack(() => initial.amount?.toString() ?? ''));
	let isExpense = $state(!isNaN(val) ? val < 0 : true);

	const hasInitialTimestamp = untrack(() => !!initial.created_at);
	let timestamp = $state(
		untrack(() => (initial.created_at ? formatTimestampLocal(initial.created_at) : ''))
	);

	type CategoryWithPath = Category & { pathName?: string };

	let categories = $state<CategoryWithPath[]>([]);
	let inputValue = $state('');
	let selectedCategory = $state<Category | null>(null);
	let showDropdown = $state(false);
	let error = $state('');
	let categoryContainer: HTMLDivElement | undefined = $state();

	let filtered = $derived(
		categories.filter((cat) =>
			(cat.pathName || cat.name).toLowerCase().includes(inputValue.toLowerCase())
		)
	);

	let timestampTouched = $state(false);
	let timer: ReturnType<typeof setInterval> | null = $state(null);

	function formatNumberString(value: string): string {
		if (!value) return '';

		let formatted = value.replace(/[^0-9.]/g, '');
		formatted = formatted.replace(/(\..*?)\./g, '$1');
		formatted = formatted.replace(/^(-?)0+(\d)/, '$1$2');

		return formatted;
	}

	function toggleExpense() {
		isExpense = !isExpense;
	}

	onMount(() => {
		async function fetchInitialData() {
			const result = await getCategories();
			const map = new Map(result.map((c) => [c.id, c]));

			const enriched = result.map((c) => {
				let path = c.name;
				let curr = c;
				while (curr.parent_id && map.has(curr.parent_id)) {
					curr = map.get(curr.parent_id)!;
					path = curr.name + ' / ' + path;
				}
				return { ...c, pathName: path };
			});

			categories = enriched;

			if (initial.category) {
				const initCat = enriched.find((c) => c.id === initial.category?.id);
				selectedCategory = initCat || initial.category;
				inputValue = initCat?.pathName || initial.category.name;
			}
		}

		fetchInitialData();

		if (!hasInitialTimestamp) {
			const updateTime = () => {
				if (!timestampTouched) {
					const now = new Date();
					const tzOffset = now.getTimezoneOffset() * 60000;
					const localISO = new Date(now.getTime() - tzOffset).toISOString().slice(0, 19);
					timestamp = localISO;
				}
			};
			updateTime();
			timer = setInterval(updateTime, 1000);
		}

		const handleClickOutside = (event: MouseEvent) => {
			if (categoryContainer && !categoryContainer.contains(event.target as Node)) {
				showDropdown = false;
			}
		};
		document.addEventListener('click', handleClickOutside, true);

		// Return a cleanup function directly from onMount
		return () => {
			document.removeEventListener('click', handleClickOutside, true);
			if (timer) clearInterval(timer);
		};
	});

	const toISOStringIfDefined = (str: string | undefined) =>
		str ? new Date(str).toISOString() : undefined;

	function handleSelect(category: CategoryWithPath) {
		selectedCategory = category;
		inputValue = category.name;
		showDropdown = false;
	}

	async function submit() {
		error = '';

		if (!amount || !selectedCategory) {
			error = 'Category, and amount are required.';
			return;
		}

		const payload: NewTransaction = {
			description,
			amount: Number(amount) * (isExpense ? -1 : 1),
			category_id: selectedCategory.id
		};

		if (timestampTouched || hasInitialTimestamp) {
			payload.created_at = toISOStringIfDefined(timestamp || undefined);
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

<form onsubmit={submit} class="space-y-4 max-w-md">
	<div>
		<label for="amnt" class="block font-medium">Amount</label>
		<input
			id="amnt"
			type="text"
			inputmode="numeric"
			bind:value={amount}
			oninput={(e) => {
				const input = e.target as HTMLInputElement;
				amount = formatNumberString(input.value);
				input.value = amount;
			}}
			class="w-full p-2 border rounded {isExpense
				? 'text-red-800 dark:text-red-200'
				: 'text-green-800 dark:text-green-200'}"
		/>
	</div>

	<div>
		<span class="font-medium {isExpense ? 'text-red-600' : 'text-gray-500 dark:text-gray-300'}"
			>Expense</span
		>
		<button
			type="button"
			role="switch"
			aria-checked={isExpense}
			aria-label="Toggle expense or income"
			onclick={toggleExpense}
			class="relative inline-flex h-6 w-12 items-center rounded-full transition-colors focus:outline-none
                {isExpense ? 'bg-red-600' : 'bg-green-600'}"
		>
			<span
				class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform
                    {isExpense ? 'translate-x-1' : 'translate-x-7'}"
			></span>
		</button>
		<span
			class="font-medium {isExpense ? 'text-gray-500 dark:text-gray-300' : 'text-green-600'}"
			>Income</span
		>
	</div>

	<div bind:this={categoryContainer}>
		<label for="cat" class="block font-medium">Category</label>
		<input
			id="cat"
			type="text"
			bind:value={inputValue}
			oninput={() => (showDropdown = true)}
			onfocus={() => (showDropdown = true)}
			placeholder="Select category..."
			class="w-full p-2 border rounded"
		/>
		{#if showDropdown}
			<ul
				class="absolute z-10 bg-white dark:bg-gray-800 border w-fit mt-1 max-h-60 overflow-auto shadow rounded"
			>
				{#each filtered as category (category.id)}
					<li class="px-3 py-2 hover:bg-gray-100 dark:hover:bg-gray-950 cursor-pointer">
						<button onclick={() => handleSelect(category)}>{category.pathName}</button>
					</li>
				{/each}
				{#if filtered.length === 0}
					<li class="px-3 py-2 text-gray-500 dark:text-gray-300">No matches found</li>
				{/if}
			</ul>
		{/if}
	</div>

	<div>
		<label for="desc" class="block font-medium">Description</label>
		<input id="desc" bind:value={description} class="w-full p-2 border rounded" />
	</div>

	<div>
		<label for="time" class="block font-medium">Timestamp</label>
		<input
			id="time"
			type="datetime-local"
			bind:value={timestamp}
			step="1"
			class="w-full p-2 border rounded"
			onfocus={handleTimestampFocusOrInput}
			oninput={handleTimestampFocusOrInput}
		/>
	</div>

	<div class="flex space-x-4">
		<button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
			{submitLabel}
		</button>
		{#if showCancel}
			<button
				type="button"
				onclick={onCancel}
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
