<script lang="ts">
	import type { Transaction, Category } from '$lib/types';
	import { SvelteMap, SvelteSet } from 'svelte/reactivity';

	let { data }: { data: { transactions: Transaction[]; categories: Category[] } } = $props();

	// --- Controls State ---
	let periodType = $state<'month' | 'year' | 'all' | 'custom'>('all');
	let periodOffset = $state(0); // 0 = current, -1 = previous, etc.
	let customStart = $state('');
	let customEnd = $state('');

	let viewMode = $state<'expenses' | 'income' | 'combined'>('combined');
	let amountMode = $state<'total' | 'average'>('total');

	let expandedIds = $state<Set<number>>(new Set());

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
			// Find earliest transaction
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
		// 1. Filter transactions by date AND view mode
		const filteredTxs = data.transactions.filter((tx) => {
			const txDate = new Date(tx.created_at);
			if (dateRange.start && txDate < dateRange.start) return false;
			if (dateRange.end && txDate > dateRange.end) return false;

			// STRICT DIRECTIONAL FILTERING:
			// Filter out opposite-sign transactions before summing for isolated views
			if (viewMode === 'expenses' && tx.amount > 0) return false;
			if (viewMode === 'income' && tx.amount < 0) return false;

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
				// Total is direct + sum of all children's totals
				node.totalSum =
					node.directSum + node.children.reduce((sum, child) => sum + child.totalSum, 0);
			});

			// Sort from most negative to most positive based on the calculated totalSum.
			// Fall back to alphabetical sorting if totals are identical.
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
		// Calculate average if toggled (no sign flipping needed anymore)
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

	<div class="flex flex-wrap gap-2">
		<select bind:value={viewMode} class="p-2 border rounded bg-white dark:bg-gray-800">
			<option value="expenses">Expenses Only</option>
			<option value="income">Income Only</option>
			<option value="combined">Net Combined</option>
		</select>

		<select bind:value={amountMode} class="p-2 border rounded bg-white dark:bg-gray-800">
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
			No transactions found for this period.
		</div>
	{/if}
</div>
