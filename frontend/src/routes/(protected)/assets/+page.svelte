<script lang="ts">
	import type { Transaction, Category, Tag } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { SvelteMap, SvelteSet } from 'svelte/reactivity';

	let { data }: { data: { transactions: Transaction[]; categories: Category[]; tags: Tag[] } } =
		$props();

	// --- Controls State ---
	let viewMode = $state<'expenses' | 'income' | 'combined'>('combined');
	let showClosed = $state(true);
	let includeNonAssetCategories = $state(true); // TCO Toggle
	let interestRate = $state<number>(0);

	// Svelte 5 Reactivity for toggles (Composite keys: tagId_categoryId)
	// eslint-disable-next-line svelte/no-unnecessary-state-wrap
	let expandedIds = $state(new SvelteSet<string>());
	// eslint-disable-next-line svelte/no-unnecessary-state-wrap
	let showTxIds = $state(new SvelteSet<string>());

	// --- Data Structures ---
	type CatNode = {
		category: Category;
		transactions: Transaction[];
		allTransactions: Transaction[];
		totalCost: number;
		monthlyCost: number;
		children: CatNode[];
	};

	type TagNode = {
		id: string;
		tag: Tag | null;
		totalCost: number;
		capexTotal: number;
		opexTotal: number;
		monthlyCost: number;
		monthsSpan: number;
		minDate: Date | null;
		rootCategories: CatNode[];
	};

	// --- Core Aggregation Engine ---
	let treeData = $derived.by(() => {
		const interest = (interestRate || 0) / 100;
		const now = new Date();
		const catMap = new Map(data.categories.map((c) => [c.id, c]));

		// 1. Group transactions by Tag, applying our filters
		const txsByTag = new SvelteMap<number | 'untagged', Transaction[]>();
		data.transactions.forEach((tx) => {
			if (viewMode === 'expenses' && tx.amount > 0) return;
			if (viewMode === 'income' && tx.amount < 0) return;

			const tags = tx.tags && tx.tags.length > 0 ? tx.tags : [null];

			tags.forEach((t) => {
				const key = t ? t.id : 'untagged';

				if (key === 'untagged') {
					if (tx.category.is_asset) {
						if (!txsByTag.has(key)) txsByTag.set(key, []);
						txsByTag.get(key)!.push(tx);
					}
				} else {
					if (includeNonAssetCategories || tx.category.is_asset) {
						if (!txsByTag.has(key)) txsByTag.set(key, []);
						txsByTag.get(key)!.push(tx);
					}
				}
			});
		});

		const nodes: TagNode[] = [];
		const untaggedNodes: TagNode[] = [];

		const msPerMonth = 1000 * 60 * 60 * 24 * 30.44;
		const msPerYear = 1000 * 60 * 60 * 24 * 365.25;

		// 2. Build Category Trees scoped per Tag
		for (const [key, txs] of txsByTag.entries()) {
			if (txs.length === 0) continue; // Optimization: Skip rendering empty trees

			const isUntagged = key === 'untagged';
			const tag = isUntagged ? null : data.tags.find((t) => t.id === key) || null;

			const endDate = tag && tag.closing_date ? new Date(tag.closing_date) : now;

			let minDate = txs[0]?.created_at ? new Date(txs[0].created_at) : now;
			txs.forEach((tx) => {
				const d = new Date(tx.created_at);
				if (d < minDate) minDate = d;
			});

			let monthsSpan = (endDate.getTime() - minDate.getTime()) / msPerMonth;
			if (monthsSpan < 1) monthsSpan = 1;

			const catNodesMap = new SvelteMap<number, CatNode>();
			let tagTotalCost = 0;
			let capexTotal = 0;
			let opexTotal = 0;

			txs.forEach((tx) => {
				const txDate = new Date(tx.created_at);
				let yearsDiff = (endDate.getTime() - txDate.getTime()) / msPerYear;
				if (yearsDiff < 0) yearsDiff = 0;

				const adjustedAmount = tx.amount * Math.pow(1 + interest, yearsDiff);
				tagTotalCost += adjustedAmount;

				// Track CapEx vs OpEx
				if (tx.category.is_asset) {
					capexTotal += adjustedAmount;
				} else {
					opexTotal += adjustedAmount;
				}

				let currentCatId: number | null = tx.category.id;

				while (currentCatId !== null && catMap.has(currentCatId)) {
					if (!catNodesMap.has(currentCatId)) {
						catNodesMap.set(currentCatId, {
							category: catMap.get(currentCatId)!,
							transactions: [],
							allTransactions: [],
							totalCost: 0,
							monthlyCost: 0,
							children: []
						});
					}
					const node = catNodesMap.get(currentCatId)!;
					node.totalCost += adjustedAmount;
					node.allTransactions.push(tx);

					if (currentCatId === tx.category.id) {
						node.transactions.push(tx);
					}

					currentCatId = catMap.get(currentCatId)!.parent_id;
				}
			});

			// 3. Assemble branches, prune useless upper levels, and sort
			let rawRoots: CatNode[] = [];
			for (const node of catNodesMap.values()) {
				node.monthlyCost = node.totalCost / monthsSpan;
				node.allTransactions.sort(
					(a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
				);

				const parentId = node.category.parent_id;
				if (parentId !== null && catNodesMap.has(parentId)) {
					catNodesMap.get(parentId)!.children.push(node);
				} else {
					rawRoots.push(node);
				}
			}

			const pruneTree = (nodesToPrune: CatNode[]): CatNode[] => {
				let pruned: CatNode[] = [];
				for (const node of nodesToPrune) {
					if (!node.category.is_asset && node.transactions.length === 0) {
						pruned.push(...pruneTree(node.children));
					} else {
						node.children = pruneTree(node.children);
						pruned.push(node);
					}
				}
				return pruned;
			};

			const sortTreeNodes = (n: CatNode[]) => {
				n.sort((a, b) => a.category.name.localeCompare(b.category.name));
				n.forEach((child) => sortTreeNodes(child.children));
			};

			const finalRoots = pruneTree(rawRoots);
			sortTreeNodes(finalRoots);

			const tagNode: TagNode = {
				id: isUntagged ? 'untagged' : tag!.id.toString(),
				tag,
				totalCost: tagTotalCost,
				capexTotal,
				opexTotal,
				monthlyCost: tagTotalCost / monthsSpan,
				monthsSpan,
				minDate,
				rootCategories: finalRoots
			};

			if (isUntagged) untaggedNodes.push(tagNode);
			else nodes.push(tagNode);
		}

		return { nodes, untagged: untaggedNodes[0] || null };
	});

	let activeAssets = $derived(
		treeData.nodes
			.filter((n) => !n.tag!.closing_date)
			.sort((a, b) => a.tag!.name.localeCompare(b.tag!.name))
	);

	let closedAssets = $derived(
		treeData.nodes
			.filter((n) => n.tag!.closing_date)
			.sort((a, b) => a.tag!.name.localeCompare(b.tag!.name))
	);

	let untaggedAsset = $derived(treeData.untagged);

	// --- Formatters & Helpers ---

	// Optimization: Instantiate once outside the formatting function
	const currencyFormatter = new Intl.NumberFormat('en-US', {
		style: 'currency',
		currency: 'USD',
		signDisplay: 'auto'
	});

	function formatCurrency(amount: number) {
		return currencyFormatter.format(amount);
	}

	function toggleExpand(compositeId: string) {
		const next = new SvelteSet(expandedIds);
		if (next.has(compositeId)) next.delete(compositeId);
		else next.add(compositeId);
		expandedIds = next;
	}

	function toggleTransactions(compositeId: string) {
		const next = new SvelteSet(showTxIds);
		if (next.has(compositeId)) next.delete(compositeId);
		else next.add(compositeId);
		showTxIds = next;
	}
</script>

<div class="mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
	<div class="flex-shrink-0">
		<h1 class="text-2xl font-bold">Asset Summary</h1>
	</div>

	<div
		class="flex flex-col sm:flex-row flex-wrap items-end sm:items-center justify-end gap-4 ml-auto"
	>
		<div class="flex items-center gap-2">
			<label
				for="interest"
				class="text-sm font-medium text-gray-600 dark:text-gray-300 cursor-help"
				title="Annualized Time Value of Money (compounded against the asset's lifespan)"
			>
				TVM Interest:
			</label>
			<div class="relative">
				<input
					id="interest"
					type="number"
					step="0.1"
					bind:value={interestRate}
					class="w-20 p-2 pr-6 border rounded bg-white dark:bg-gray-800 text-sm"
				/>
				<span class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500">%</span>
			</div>
		</div>

		<label class="flex items-center gap-2 text-sm font-medium cursor-pointer">
			<input
				type="checkbox"
				bind:checked={includeNonAssetCategories}
				class="rounded text-blue-600"
			/>
			Include All Tagged Transactions
		</label>

		<label class="flex items-center gap-2 text-sm font-medium cursor-pointer">
			<input type="checkbox" bind:checked={showClosed} class="rounded text-blue-600" />
			Show Closed
		</label>

		<select bind:value={viewMode} class="p-2 border rounded bg-white dark:bg-gray-800 text-sm">
			<option value="combined">Net Combined</option>
			<option value="expenses">Expenses Only</option>
			<option value="income">Income Only</option>
		</select>
	</div>
</div>

{#snippet treeRow(node: CatNode, tagId: string, depth: number)}
	{@const compositeId = `${tagId}_${node.category.id}`}
	{@const isExpanded = expandedIds.has(compositeId)}
	{@const txsVisible = showTxIds.has(compositeId)}
	{@const hasChildren = node.children.length > 0}
	{@const hasTransactions = node.allTransactions.length > 0}

	<div
		class="flex items-center justify-between py-3 border-b dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors bg-white dark:bg-gray-900"
	>
		<div class="flex items-center flex-1 min-w-0" style="padding-left: {depth * 1.5 + 1}rem;">
			{#if hasChildren}
				<button
					onclick={() => toggleExpand(compositeId)}
					aria-label="{isExpanded ? 'Collapse' : 'Expand'} {node.category.name}"
					aria-expanded={isExpanded}
					class="w-6 h-6 flex items-center justify-center text-gray-500 hover:text-gray-800 dark:hover:text-white mr-1"
				>
					{isExpanded ? '▼' : '▶'}
				</button>
			{:else}
				<div class="w-7 h-6"></div>
			{/if}

			<div class="flex flex-col sm:flex-row sm:items-center gap-1 sm:gap-3 truncate">
				<span
					class="font-medium {depth === 0
						? 'text-lg'
						: ''} truncate text-gray-800 dark:text-gray-200"
				>
					{node.category.name}
				</span>
				{#if hasTransactions}
					<button
						onclick={() => toggleTransactions(compositeId)}
						class="text-left text-xs text-blue-600 dark:text-blue-400 hover:underline font-medium"
					>
						{txsVisible ? 'Hide' : 'Show'} Transactions ({node.allTransactions.length})
					</button>
				{/if}
			</div>
		</div>

		<div
			class="flex flex-col sm:flex-row items-end sm:items-center gap-1 sm:gap-6 text-right pr-4"
		>
			<div
				class="font-bold text-base {node.monthlyCost < 0
					? 'text-red-600 dark:text-red-400'
					: 'text-green-600 dark:text-green-400'}"
			>
				<span class="whitespace-nowrap">{formatCurrency(node.monthlyCost)}</span><span
					class="text-xs font-normal text-gray-500">/mo</span
				>
			</div>
			<div
				class="font-bold text-base w-auto sm:w-28 {node.totalCost < 0
					? 'text-red-600 dark:text-red-400'
					: 'text-green-600 dark:text-green-400'}"
			>
				<span class="whitespace-nowrap">{formatCurrency(node.totalCost)}</span>
			</div>
		</div>
	</div>

	{#if txsVisible && hasTransactions}
		<div
			class="bg-gray-50 dark:bg-gray-800/40 border-b border-dashed dark:border-gray-700 py-3 transition-all"
			style="padding-left: {(depth + 1.2) * 1.5 + 1}rem; padding-right: 1rem;"
		>
			<h4 class="text-xs font-bold text-gray-400 uppercase tracking-wider mb-2">
				Rollup Ledger ({node.category.name})
			</h4>
			<ul class="space-y-1.5 max-h-64 overflow-y-auto pr-2">
				{#each node.allTransactions as tx (tx.id)}
					<li
						class="flex justify-between items-center text-sm bg-white dark:bg-gray-800 px-3 py-1.5 rounded shadow-sm border border-gray-100 dark:border-gray-700"
					>
						<div class="flex items-center min-w-0 flex-1 pr-4">
							<span
								class="text-xs font-mono text-gray-400 bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded mr-3 whitespace-nowrap"
							>
								{formatTimestampLocalForDisplay(tx.created_at).split(',')[0]}
							</span>
							<div
								class="truncate flex flex-col sm:flex-row sm:items-center sm:gap-2"
							>
								<span class="text-gray-700 dark:text-gray-300 font-medium truncate"
									>{tx.description}</span
								>
								{#if tx.category.id !== node.category.id}
									<span
										class="text-[10px] text-gray-400 dark:text-gray-500 italic bg-gray-50 dark:bg-gray-900 px-1 rounded border dark:border-gray-700"
									>
										via {tx.category.name}
									</span>
								{/if}
							</div>
						</div>
						<span
							class="font-semibold whitespace-nowrap {tx.amount < 0
								? 'text-red-500'
								: 'text-green-500'}"
						>
							{formatCurrency(tx.amount)}
						</span>
					</li>
				{/each}
			</ul>
		</div>
	{/if}

	{#if isExpanded && hasChildren}
		{#each node.children as child (child.category.id)}
			{@render treeRow(child, tagId, depth + 1)}
		{/each}
	{/if}
{/snippet}

{#snippet tagSection(node: TagNode, isClosed: boolean, isUntagged: boolean)}
	{@const totalAbs = Math.abs(node.capexTotal) + Math.abs(node.opexTotal)}
	{@const capexPct =
		totalAbs !== 0 ? Math.round((Math.abs(node.capexTotal) / totalAbs) * 100) : 0}
	{@const opexPct = totalAbs !== 0 ? Math.round((Math.abs(node.opexTotal) / totalAbs) * 100) : 0}

	<details
		class="bg-gray-100 dark:bg-gray-800 rounded shadow overflow-hidden group {isClosed &&
		!isUntagged
			? 'opacity-75 hover:opacity-100 transition-opacity'
			: ''} {isUntagged ? 'border border-red-300 dark:border-red-800' : ''}"
	>
		<summary
			class="flex flex-col sm:flex-row sm:items-center justify-between p-4 cursor-pointer hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors list-none"
		>
			<div class="flex items-center gap-2">
				<span class="text-gray-400 group-open:rotate-90 transition-transform">▶</span>
				<span
					class="font-bold text-xl {isClosed
						? 'line-through text-gray-500'
						: ''} {isUntagged ? 'text-red-600 dark:text-red-400' : ''}"
				>
					{node.tag?.name || 'Missing Tags'}
				</span>
			</div>

			<div class="flex flex-col items-end mt-2 sm:mt-0 text-right">
				<div class="flex items-center gap-4 sm:gap-6">
					<div
						class="font-bold text-lg {node.monthlyCost < 0
							? 'text-red-600 dark:text-red-400'
							: 'text-green-600 dark:text-green-400'}"
					>
						<span class="whitespace-nowrap">{formatCurrency(node.monthlyCost)}</span
						><span class="text-sm font-normal text-gray-500">/mo</span>
					</div>
					<div
						class="font-bold text-lg {node.totalCost < 0
							? 'text-red-600 dark:text-red-400'
							: 'text-green-600 dark:text-green-400'}"
					>
						Total: <span class="whitespace-nowrap"
							>{formatCurrency(node.totalCost)}</span
						>
					</div>
				</div>

				<div
					class="flex flex-wrap justify-end items-center gap-x-2 gap-y-1 mt-1 text-xs text-gray-500 dark:text-gray-400 font-medium"
				>
					<span>CapEx: {formatCurrency(node.capexTotal)} ({capexPct}%)</span>
					<span>•</span>
					<span>OpEx: {formatCurrency(node.opexTotal)} ({opexPct}%)</span>
					<span>•</span>
					<span>(over {node.monthsSpan.toFixed(1)} mos)</span>
				</div>
			</div>
		</summary>

		<div class="border-t dark:border-gray-700 bg-white dark:bg-gray-900">
			{#each node.rootCategories as rootNode (rootNode.category.id)}
				{@render treeRow(rootNode, node.id, 0)}
			{/each}
		</div>
	</details>
{/snippet}

<div class="space-y-8">
	{#if activeAssets.length > 0}
		<section>
			<h2 class="text-lg font-semibold mb-3 border-b pb-1">Active Assets</h2>
			<div class="space-y-4">
				{#each activeAssets as node (node.id)}
					{@render tagSection(node, false, false)}
				{/each}
			</div>
		</section>
	{/if}

	{#if showClosed && closedAssets.length > 0}
		<section>
			<h2 class="text-lg font-semibold mb-3 border-b pb-1 text-gray-500">Closed Assets</h2>
			<div class="space-y-4">
				{#each closedAssets as node (node.id)}
					{@render tagSection(node, true, false)}
				{/each}
			</div>
		</section>
	{/if}

	{#if untaggedAsset && Math.abs(untaggedAsset.totalCost) > 0.01}
		<section>
			<h2 class="text-lg font-semibold mb-3 border-b pb-1 text-red-500">Untagged Assets</h2>
			<div class="space-y-4">
				{@render tagSection(untaggedAsset, false, true)}
			</div>
		</section>
	{/if}

	{#if activeAssets.length === 0 && closedAssets.length === 0 && !untaggedAsset}
		<div class="p-8 text-center text-gray-500 italic bg-gray-50 dark:bg-gray-800 rounded">
			No asset transactions found. Tag your transactions in asset categories to see them here!
		</div>
	{/if}
</div>
