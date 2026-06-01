<script lang="ts">
	import type { Transaction, Category } from '$lib/types';
	import { formatTimestampLocalForDisplay } from '$lib/utils';
	import { SvelteMap, SvelteSet } from 'svelte/reactivity';

	let { data }: { data: { transactions: Transaction[]; categories: Category[] } } = $props();

	// Separate states for category tree nesting vs transaction visibility
	let expandedIds = $state<Set<number>>(new Set());
	let showTxIds = $state<Set<number>>(new Set());

	type AssetNode = {
		category: Category;
		allTransactions: Transaction[]; // Includes self + ALL descendants (whether asset or not)
		totalCost: number;
		monthsOwned: number;
		monthlyCost: number;
		children: AssetNode[];
	};

	let treeData = $derived.by(() => {
		const allCatsMap = new Map(data.categories.map((c) => [c.id, c]));
		const assetCategories = data.categories.filter((c) => c.is_asset);
		const assetNodesMap = new SvelteMap<number, AssetNode>();

		// Helper to recursively pull all descendant category IDs (asset and non-asset alike)
		const getDescendantIds = (catId: number): number[] => {
			const ids: number[] = [];
			const queue = [catId];
			while (queue.length > 0) {
				const currentId = queue.shift()!;
				const children = data.categories.filter((c) => c.parent_id === currentId);
				for (const child of children) {
					ids.push(child.id);
					queue.push(child.id);
				}
			}
			return ids;
		};

		// 1. Initialize asset nodes and compute their complete scoped math
		assetCategories.forEach((cat) => {
			const scopeIds = [cat.id, ...getDescendantIds(cat.id)];

			// Gather transactions belonging to this category or any child branch
			const scopedTxs = data.transactions
				.filter((tx) => scopeIds.includes(tx.category.id))
				.sort(
					(a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
				);

			// Compute custom math metrics
			const totalCost = scopedTxs.reduce((sum, tx) => sum + tx.amount, 0);
			let monthsOwned = 1;

			if (scopedTxs.length > 0) {
				const earliestTx = scopedTxs.reduce(
					(min, tx) => (tx.created_at < min ? tx.created_at : min),
					scopedTxs[0].created_at
				);
				const msDiff = new Date().getTime() - new Date(earliestTx).getTime();
				monthsOwned = Math.max(1, msDiff / (1000 * 60 * 60 * 24 * 30.44));
			}

			assetNodesMap.set(cat.id, {
				category: cat,
				allTransactions: scopedTxs,
				totalCost,
				monthsOwned,
				monthlyCost: totalCost / monthsOwned,
				children: []
			});
		});

		// 2. Build the pruned tree structure by searching for closest asset ancestors
		const roots: AssetNode[] = [];

		assetCategories.forEach((cat) => {
			const node = assetNodesMap.get(cat.id)!;
			let currentParentId = cat.parent_id;
			let nearestAssetParent: AssetNode | null = null;

			// Traverse up the tree until we find a parent that is also marked as an asset
			while (currentParentId !== null) {
				if (assetNodesMap.has(currentParentId)) {
					nearestAssetParent = assetNodesMap.get(currentParentId)!;
					break;
				}
				const parentCat = allCatsMap.get(currentParentId);
				currentParentId = parentCat ? parentCat.parent_id : null;
			}

			if (nearestAssetParent) {
				nearestAssetParent.children.push(node);
			} else {
				roots.push(node);
			}
		});

		// 3. Alphabetize the sorted tree outputs
		const sortTreeNodes = (nodes: AssetNode[]) => {
			nodes.sort((a, b) => a.category.name.localeCompare(b.category.name));
			nodes.forEach((n) => sortTreeNodes(n.children));
		};

		sortTreeNodes(roots);
		return roots;
	});

	function formatCurrency(amount: number) {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD',
			signDisplay: 'auto'
		}).format(amount);
	}

	function toggleExpand(id: number) {
		const next = new SvelteSet(expandedIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		expandedIds = next;
	}

	function toggleTransactions(id: number) {
		const next = new SvelteSet(showTxIds);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		showTxIds = next;
	}

	function expandAll() {
		expandedIds = new Set(data.categories.filter((c) => c.is_asset).map((c) => c.id));
	}

	function collapseAll() {
		expandedIds = new Set();
		showTxIds = new Set();
	}
</script>

<div class="mb-6 flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
	<h1 class="text-2xl font-bold">Asset Cost Tracker</h1>
</div>

<div class="flex gap-2 mb-2">
	<button onclick={expandAll} class="text-sm text-blue-600 dark:text-blue-400 hover:underline">
		Expand All Nodes
	</button>
	<span class="text-gray-400">|</span>
	<button onclick={collapseAll} class="text-sm text-blue-600 dark:text-blue-400 hover:underline">
		Collapse All
	</button>
</div>

{#snippet treeRow(node: AssetNode, depth: number)}
	{@const isExpanded = expandedIds.has(node.category.id)}
	{@const txsVisible = showTxIds.has(node.category.id)}
	{@const hasChildren = node.children.length > 0}
	{@const hasTransactions = node.allTransactions.length > 0}

	<div
		class="flex items-center justify-between py-3 border-b dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
	>
		<div class="flex items-center flex-1 min-w-0" style="padding-left: {depth * 1.5}rem;">
			{#if hasChildren}
				<button
					onclick={() => toggleExpand(node.category.id)}
					class="w-6 h-6 flex items-center justify-center text-gray-500 hover:text-gray-800 dark:hover:text-white mr-1"
				>
					{isExpanded ? '▼' : '▶'}
				</button>
			{:else}
				<div class="w-7 h-6"></div>
			{/if}

			<div class="flex flex-col sm:flex-row sm:items-center gap-1 sm:gap-3 truncate">
				<span class="font-medium {depth === 0 ? 'text-lg' : ''} truncate">
					{node.category.name}
				</span>
				{#if hasTransactions}
					<button
						onclick={() => toggleTransactions(node.category.id)}
						class="text-left text-xs text-blue-600 dark:text-blue-400 hover:underline font-medium"
					>
						{txsVisible ? 'Hide' : 'Show'} Transactions ({node.allTransactions.length})
					</button>
				{/if}
			</div>
		</div>

		<div class="flex flex-col items-end text-right ml-4">
			<div
				class="font-bold text-lg {node.monthlyCost < 0
					? 'text-red-600 dark:text-red-400'
					: 'text-green-600 dark:text-green-400'}"
			>
				{formatCurrency(node.monthlyCost)}
				<span class="text-sm font-normal text-gray-500">/mo</span>
			</div>
			<div class="text-xs text-gray-500 font-medium">
				Total: {formatCurrency(node.totalCost)} (over {node.monthsOwned.toFixed(1)} mos)
			</div>
		</div>
	</div>

	{#if txsVisible && hasTransactions}
		<div
			class="bg-gray-50 dark:bg-gray-800/40 border-b border-dashed dark:border-gray-700 py-3 transition-all"
			style="padding-left: {(depth + 1.2) * 1.5}rem; padding-right: 1.5rem;"
		>
			<h4 class="text-xs font-bold text-gray-400 uppercase tracking-wider mb-2">
				Rollup Ledger ({node.category.name})
			</h4>
			<ul class="space-y-1.5 max-h-64 overflow-y-auto pr-2">
				{#each node.allTransactions as tx (tx)}
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
			{@render treeRow(child, depth + 1)}
		{/each}
	{/if}
{/snippet}

<div class="bg-white dark:bg-gray-900 rounded shadow overflow-hidden">
	{#each treeData as rootNode (rootNode.category.id)}
		{@render treeRow(rootNode, 0)}
	{/each}

	{#if treeData.length === 0}
		<div class="p-8 text-center text-gray-500 italic">
			No asset categories found. Edit your categories to track them as assets.
		</div>
	{/if}
</div>
