<script lang="ts">
	import { untrack } from 'svelte';
	import type { Transaction, Category, Tag } from '$lib/types';
	import { SvelteMap, SvelteSet } from 'svelte/reactivity';

	let { data }: { data: { transactions: Transaction[]; categories: Category[]; tags: Tag[] } } =
		$props();

	// --- Controls State ---
	let periodType = $state<'month' | 'year' | 'all' | 'custom'>('all');
	let periodOffset = $state(0); // 0 = current, -1 = previous, etc.
	let customStart = $state('');
	let customEnd = $state('');

	let viewMode = $state<'expenses' | 'income' | 'combined'>('combined');
	let amountMode = $state<'total' | 'average'>('total');

	let expandedIds = $state<Set<number>>(new Set());

	// --- Tag Filter Logic ---
	let sortedTags = $derived([...data.tags].sort((a, b) => a.name.localeCompare(b.name)));

	let selectedTagIds = $state<Set<number>>(untrack(() => new Set(sortedTags.map((t) => t.id))));

	let isTagFilterActive = $derived(selectedTagIds.size < sortedTags.length);
	let isTagFilterOpen = $state(false);
	let tagFilterContainer: HTMLDivElement | undefined = $state();
	let tagSearchQuery = $state('');

	let visibleTags = $derived(
		sortedTags.filter((tag) => tag.name.toLowerCase().includes(tagSearchQuery.toLowerCase()))
	);

	function toggleTag(id: number) {
		const newSet = new SvelteSet(selectedTagIds);
		if (newSet.has(id)) newSet.delete(id);
		else newSet.add(id);
		selectedTagIds = newSet;
	}

	function selectAllTags() {
		selectedTagIds = new Set(sortedTags.map((t) => t.id));
	}

	function clearAllTags() {
		selectedTagIds = new Set();
	}

	$effect(() => {
		function handleClickOutside(event: MouseEvent) {
			if (tagFilterContainer && !tagFilterContainer.contains(event.target as Node)) {
				isTagFilterOpen = false;
			}
		}
		document.addEventListener('click', handleClickOutside);
		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});

	// --- Date Math ---
	let dateRange = $derived.by(() => {
		const now = new Date();
		if (periodType === 'all') return { start: null, end: null };
		if (periodType === 'custom')
			return {
				start: customStart ? new Date(customStart) : null,
				end: customEnd ? new Date(customEnd + 'T23:59:59') : null
			};

		if (periodType === 'month') {
			const start = new Date(now.getFullYear(), now.getMonth() + periodOffset, 1);
			const end = new Date(
				now.getFullYear(),
				now.getMonth() + periodOffset + 1,
				0,
				23,
				59,
				59
			);
			return { start, end };
		}

		if (periodType === 'year') {
			const start = new Date(now.getFullYear() + periodOffset, 0, 1);
			const end = new Date(now.getFullYear() + periodOffset, 11, 31, 23, 59, 59);
			return { start, end };
		}
		return { start: null, end: null };
	});

	let dateLabel = $derived.by(() => {
		if (periodType === 'all') return 'All Time';
		if (periodType === 'custom') return 'Custom Range';
		if (!dateRange.start) return '';

		if (periodType === 'month') {
			return dateRange.start.toLocaleDateString(undefined, {
				month: 'long',
				year: 'numeric'
			});
		}
		if (periodType === 'year') {
			return dateRange.start.getFullYear().toString();
		}
		return '';
	});

	let monthsSpan = $derived.by(() => {
		if (periodType === 'month') return 1;
		if (periodType === 'year') return 12;

		let start = dateRange.start;
		let end = dateRange.end || new Date();

		if (periodType === 'all' && data.transactions.length > 0) {
			const earliest = data.transactions.reduce(
				(min, tx) => (tx.created_at < min ? tx.created_at : min),
				data.transactions[0].created_at
			);
			start = new Date(earliest);
		}

		if (!start) return 1;
		const msDiff = end.getTime() - start.getTime();
		const months = msDiff / (1000 * 60 * 60 * 24 * 30.44); // Approx days in month
		return Math.max(1, months);
	});

	// --- Tree Aggregation ---
	type TreeNode = {
		category: Category;
		directSum: number;
		totalSum: number;
		children: TreeNode[];
	};

	let treeData = $derived.by(() => {
		// 1. Filter transactions by date, view mode, AND tag
		const filteredTxs = data.transactions.filter((tx) => {
			const txDate = new Date(tx.created_at);
			if (dateRange.start && txDate < dateRange.start) return false;
			if (dateRange.end && txDate > dateRange.end) return false;

			if (viewMode === 'expenses' && tx.amount > 0) return false;
			if (viewMode === 'income' && tx.amount < 0) return false;

			// Tag Match Logic
			const tagMatch =
				!isTagFilterActive ||
				(tx.tags && tx.tags.some((tag) => selectedTagIds.has(tag.id)));
			if (!tagMatch) return false;

			return true;
		});

		// 2. Initialize node maps
		const nodeMap = new SvelteMap<number, TreeNode>();
		data.categories.forEach((cat) => {
			nodeMap.set(cat.id, { category: cat, directSum: 0, totalSum: 0, children: [] });
		});

		// 3. Sum direct transactions
		filteredTxs.forEach((tx) => {
			const node = nodeMap.get(tx.category.id);
			if (node) node.directSum += tx.amount;
		});

		// 4. Build Tree & calculate bottom-up totals
		const roots: TreeNode[] = [];

		const buildAndRollup = (parentId: number | null): TreeNode[] => {
			const childrenNodes = data.categories
				.filter((c) => c.parent_id === parentId)
				.map((c) => nodeMap.get(c.id)!);

			childrenNodes.forEach((node) => {
				node.children = buildAndRollup(node.category.id);
				node.totalSum =
					node.directSum + node.children.reduce((sum, child) => sum + child.totalSum, 0);
			});

			return childrenNodes.sort((a, b) => {
				if (a.totalSum !== b.totalSum) {
					return a.totalSum - b.totalSum;
				}
				return a.category.name.localeCompare(b.category.name);
			});
		};

		roots.push(...buildAndRollup(null));
		return roots;
	});

	// --- Formatters & Helpers ---
	function formatAmount(amount: number) {
		const finalValue = amountMode === 'average' ? amount / monthsSpan : amount;

		return (
			new Intl.NumberFormat('en-US', {
				style: 'currency',
				currency: 'USD',
				signDisplay: 'auto'
			}).format(finalValue) + (amountMode === 'average' ? '/mo' : '')
		);
	}

	function toggleExpand(id: number) {
		const next = new SvelteSet(expandedIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		expandedIds = next;
	}

	function expandAll() {
		expandedIds = new Set(data.categories.map((c) => c.id));
	}

	function collapseAll() {
		expandedIds = new Set();
	}

	function changeOffset(delta: number) {
		periodOffset += delta;
	}
</script>

<div class="mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
	<h1 class="text-2xl font-bold">Category Summary</h1>

	<div class="flex flex-wrap items-center gap-2">
		<div class="relative" bind:this={tagFilterContainer}>
			<button
				type="button"
				class="px-4 py-2 rounded flex items-center gap-2 transition-colors
                {isTagFilterActive
					? 'bg-amber-200 text-amber-900 hover:bg-amber-300 dark:bg-amber-700 dark:text-amber-100 dark:hover:bg-amber-800'
					: 'bg-gray-200 text-gray-900 hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600'}"
				onclick={() => (isTagFilterOpen = !isTagFilterOpen)}
			>
				<span class="font-medium">
					{isTagFilterActive ? 'Filter Tags (Active)' : 'Filter Tags'}
				</span>
				<svg
					class="w-4 h-4"
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M19 9l-7 7-7-7"
					/>
				</svg>
			</button>

			{#if isTagFilterOpen}
				<div
					class="absolute right-0 mt-2 w-64 bg-gray-100 dark:bg-gray-800 p-4 rounded shadow-lg z-20"
				>
					<input
						type="text"
						placeholder="Search tags..."
						class="w-full p-2 mb-3 border rounded dark:bg-gray-700 dark:text-white"
						bind:value={tagSearchQuery}
					/>

					<div class="flex flex-col gap-2 max-h-64 overflow-auto">
						{#if visibleTags.length > 0}
							{#each visibleTags as tag (tag.id)}
								<label class="flex items-center space-x-2">
									<input
										type="checkbox"
										checked={selectedTagIds.has(tag.id)}
										onchange={() => toggleTag(tag.id)}
									/>
									<span>{tag.name}</span>
								</label>
							{/each}
						{:else}
							<p class="text-sm text-gray-500 italic">No tags found</p>
						{/if}
					</div>

					<div class="flex justify-between mt-3">
						<button
							type="button"
							class="px-3 py-1 rounded bg-blue-500 text-white hover:bg-blue-600"
							onclick={selectAllTags}>Select All</button
						>
						<button
							type="button"
							class="px-3 py-1 rounded bg-gray-400 text-white hover:bg-gray-500"
							onclick={clearAllTags}>Deselect All</button
						>
					</div>
				</div>
			{/if}
		</div>

		<select bind:value={viewMode} class="p-2 border rounded bg-white dark:bg-gray-800 text-sm">
			<option value="expenses">Expenses Only</option>
			<option value="income">Income Only</option>
			<option value="combined">Net Combined</option>
		</select>

		<select
			bind:value={amountMode}
			class="p-2 border rounded bg-white dark:bg-gray-800 text-sm"
		>
			<option value="total">Total Sum</option>
			<option value="average">Monthly Average</option>
		</select>
	</div>
</div>

<div
	class="bg-gray-100 dark:bg-gray-800 p-4 rounded mb-6 flex flex-col sm:flex-row items-center gap-4 shadow-sm"
>
	<select
		bind:value={periodType}
		onchange={() => (periodOffset = 0)}
		class="p-2 border rounded bg-white dark:bg-gray-700"
	>
		<option value="month">Month</option>
		<option value="year">Year</option>
		<option value="all">All Time</option>
		<option value="custom">Custom</option>
	</select>

	{#if periodType === 'custom'}
		<div class="flex items-center gap-2">
			<input
				type="date"
				bind:value={customStart}
				class="p-2 border rounded bg-white dark:bg-gray-700"
			/>
			<span>to</span>
			<input
				type="date"
				bind:value={customEnd}
				class="p-2 border rounded bg-white dark:bg-gray-700"
			/>
		</div>
	{:else if periodType !== 'all'}
		<div class="flex items-center gap-4 font-bold text-lg">
			<button
				onclick={() => changeOffset(-1)}
				class="px-3 py-1 bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
				>←</button
			>
			<span class="w-32 text-center">{dateLabel}</span>
			<button
				onclick={() => changeOffset(1)}
				class="px-3 py-1 bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
				>→</button
			>
		</div>
	{/if}
</div>

<div class="flex gap-2 mb-2">
	<button onclick={expandAll} class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
		>Expand All</button
	>
	<span class="text-gray-400">|</span>
	<button onclick={collapseAll} class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
		>Collapse All</button
	>
</div>

{#snippet treeRow(node: TreeNode, depth: number)}
	{@const isExpanded = expandedIds.has(node.category.id)}
	{@const hasChildren = node.children.length > 0}

	{@const netValue = node.totalSum}
	{@const showInExpenses = viewMode === 'expenses' && netValue < -0.01}
	{@const showInIncome = viewMode === 'income' && netValue > 0.01}
	{@const showInCombined = viewMode === 'combined' && Math.abs(netValue) > 0.01}

	{#if showInExpenses || showInIncome || showInCombined}
		<div
			class="flex items-center justify-between py-2 border-b dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
		>
			<div class="flex items-center" style="padding-left: {depth * 1.5}rem;">
				{#if hasChildren}
					<button
						onclick={() => toggleExpand(node.category.id)}
						class="w-6 h-6 flex items-center justify-center text-gray-500 hover:text-gray-800 dark:hover:text-white"
					>
						{isExpanded ? '▼' : '▶'}
					</button>
				{:else}
					<div class="w-6 h-6"></div>
				{/if}
				<span class="font-medium {depth === 0 ? 'text-lg' : ''}">{node.category.name}</span>
			</div>

			<div
				class="font-bold {netValue < 0
					? 'text-red-600 dark:text-red-400'
					: 'text-green-600 dark:text-green-400'}"
			>
				{formatAmount(node.totalSum)}
			</div>
		</div>

		{#if isExpanded && hasChildren}
			{#if Math.abs(node.directSum) > 0.01}
				<div
					class="flex items-center justify-between py-2 border-b border-dashed dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50"
				>
					<div
						class="flex items-center text-gray-500 italic"
						style="padding-left: {(depth + 1) * 1.5}rem;"
					>
						<div class="w-6 h-6"></div>
						<span>↳ Direct ({node.category.name})</span>
					</div>
					<div
						class="text-sm italic {node.directSum < 0
							? 'text-red-500'
							: 'text-green-500'}"
					>
						{formatAmount(node.directSum)}
					</div>
				</div>
			{/if}

			{#each node.children as child (child)}
				{@render treeRow(child, depth + 1)}
			{/each}
		{/if}
	{/if}
{/snippet}

<div class="bg-white dark:bg-gray-900 rounded shadow overflow-hidden">
	{#each treeData as rootNode (rootNode)}
		{@render treeRow(rootNode, 0)}
	{/each}
	{#if treeData.length === 0 || data.transactions.length === 0}
		<div class="p-4 text-center text-gray-500 italic">
			No transactions found for this period/filter.
		</div>
	{/if}
</div>
