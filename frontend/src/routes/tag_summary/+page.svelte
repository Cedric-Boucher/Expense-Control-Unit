<script lang="ts">
	import type { Transaction, Tag } from '$lib/types';
	import { SvelteMap } from 'svelte/reactivity';

	let { data }: { data: { transactions: Transaction[]; tags: Tag[] } } = $props();

	// --- Controls State ---
	let periodType = $state<'month' | 'year' | 'all' | 'custom'>('all');
	let periodOffset = $state(0);
	let customStart = $state('');
	let customEnd = $state('');

	let viewMode = $state<'expenses' | 'income' | 'combined'>('combined');
	let amountMode = $state<'total' | 'average'>('total');

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
		const months = msDiff / (1000 * 60 * 60 * 24 * 30.44);
		return Math.max(1, months);
	});

	// --- Data Aggregation ---
	type TagSummaryRow = {
		id: number | null; // null for 'Untagged'
		name: string;
		sum: number;
	};

	let flatTagData = $derived.by(() => {
		// 1. Filter transactions by date and view mode
		const filteredTxs = data.transactions.filter((tx) => {
			const txDate = new Date(tx.created_at);
			if (dateRange.start && txDate < dateRange.start) return false;
			if (dateRange.end && txDate > dateRange.end) return false;

			if (viewMode === 'expenses' && tx.amount > 0) return false;
			if (viewMode === 'income' && tx.amount < 0) return false;

			return true;
		});

		// 2. Initialize maps
		const tagMap = new SvelteMap<number, TagSummaryRow>();
		data.tags.forEach((tag) => {
			tagMap.set(tag.id, { id: tag.id, name: tag.name, sum: 0 });
		});
		let untaggedSum = 0;

		// 3. Accumulate amounts
		// NOTE: Because a transaction can have multiple tags, its amount will be
		// added to multiple rows. This is expected behavior for tag summaries.
		filteredTxs.forEach((tx) => {
			if (!tx.tags || tx.tags.length === 0) {
				untaggedSum += tx.amount;
			} else {
				tx.tags.forEach((t) => {
					const row = tagMap.get(t.id);
					if (row) row.sum += tx.amount;
				});
			}
		});

		const rows: TagSummaryRow[] = Array.from(tagMap.values());
		if (Math.abs(untaggedSum) > 0.01) {
			rows.push({ id: null, name: 'Untagged', sum: untaggedSum });
		}

		// 4. Filter empty and sort
		const finalRows = rows.filter((r) => {
			if (viewMode === 'expenses' && r.sum > -0.01) return false;
			if (viewMode === 'income' && r.sum < 0.01) return false;
			if (viewMode === 'combined' && Math.abs(r.sum) < 0.01) return false;
			return true;
		});

		return finalRows.sort((a, b) => {
			if (a.sum !== b.sum) return a.sum - b.sum;
			return a.name.localeCompare(b.name);
		});
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

	function changeOffset(delta: number) {
		periodOffset += delta;
	}
</script>

<div class="mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
	<div>
		<h1 class="text-2xl font-bold">Tag Summary</h1>
	</div>

	<div class="flex flex-wrap gap-2">
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

<div class="bg-white dark:bg-gray-900 rounded shadow overflow-hidden">
	{#each flatTagData as row (row.id ?? 'untagged')}
		<div
			class="flex items-center justify-between p-4 border-b dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
		>
			<div class="flex items-center gap-2">
				{#if row.id === null}
					<span class="font-medium text-gray-500 italic">{row.name}</span>
				{:else}
					<span class="font-medium text-lg">{row.name}</span>
				{/if}
			</div>

			<div
				class="font-bold {row.sum < 0
					? 'text-red-600 dark:text-red-400'
					: 'text-green-600 dark:text-green-400'}"
			>
				{formatAmount(row.sum)}
			</div>
		</div>
	{/each}

	{#if flatTagData.length === 0 || data.transactions.length === 0}
		<div class="p-4 text-center text-gray-500 italic">
			No transactions found for this period.
		</div>
	{/if}
</div>
