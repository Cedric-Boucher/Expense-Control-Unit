import { format } from 'date-fns';
import { getTransactions, getCategories, getTags, uploadUserData } from '$lib/api';
import type {
	Category,
	CategoryNode,
	Transaction,
	Tag,
	ImportPayload,
	ImportCategory,
	ImportTransaction,
	ImportTag
} from '$lib/types';
import { invalidateAll } from '$app/navigation';

export function formatTimestampLocal(isoString: string): string {
	const date = new Date(isoString);
	return format(date, "yyyy-MM-dd'T'HH:mm:ss");
}

export function formatTimestampLocalForDisplay(isoString: string): string {
	const date = new Date(isoString);
	return format(date, 'yyyy-MM-dd HH:mm:ss');
}

export async function exportUserDataToFile() {
	try {
		const [transactions, categories, tags]: [Transaction[], Category[], Tag[]] =
			await Promise.all([getTransactions(), getCategories(), getTags()]);

		// Map for instant O(1) lookups
		const categoryMap = new Map<number, Category>(categories.map((c) => [c.id, c]));
		const categoryIdToPath = new Map<number, string[]>();

		// Recursive helper to build paths and cache them
		const getPath = (id: number): string[] => {
			if (categoryIdToPath.has(id)) return categoryIdToPath.get(id)!;

			const cat = categoryMap.get(id);
			if (!cat) return []; // Should never happen unless DB data is orphaned

			const path = cat.parent_id ? [...getPath(cat.parent_id), cat.name] : [cat.name];

			categoryIdToPath.set(id, path);
			return path;
		};

		// Pre-compute all paths
		categories.forEach((cat) => getPath(cat.id));

		// Format categories for export using the new ImportCategory schema
		const exportCategories: ImportCategory[] = categories.map((cat) => ({
			path: categoryIdToPath.get(cat.id) || [],
			created_at: cat.created_at,
			is_asset: cat.is_asset
		}));

		// Format tags for export using the new ImportTag schema
		const exportTags: ImportTag[] = tags.map((tag) => ({
			name: tag.name,
			created_at: tag.created_at
		}));

		// Format transactions for export using the new ImportTransaction schema
		const exportTransactions: ImportTransaction[] = transactions.map((tx) => ({
			category_path: categoryIdToPath.get(tx.category.id) || [tx.category.name],
			amount: tx.amount,
			description: tx.description,
			created_at: tx.created_at,
			tags: tx.tags.map((t) => t.name)
		}));

		const exportData: ImportPayload = {
			categories: exportCategories,
			tags: exportTags,
			transactions: exportTransactions
		};

		const json = JSON.stringify(exportData, null, 2);

		const blob = new Blob([json], { type: 'application/json' });
		const url = URL.createObjectURL(blob);

		const a = document.createElement('a');
		a.href = url;
		a.download = `ECU-export-${new Date().toISOString()}.json`;
		document.body.appendChild(a);
		a.click();

		// Cleanup
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	} catch (error) {
		console.error('Failed to export data:', error);
		alert('Failed to export data. Please try again.');
	}
}

export async function importUserDataFromFile() {
	const input = document.createElement('input');
	input.type = 'file';
	input.accept = 'application/json';

	input.onchange = async () => {
		const file = input.files?.[0];
		if (!file) return;

		try {
			const text = await file.text();
			const data: ImportPayload = JSON.parse(text); // Parse here so we can pass typed data to API
			await uploadUserData(data);
			await invalidateAll(); // update page to show imported data
		} catch (err) {
			console.error('Import error:', err);
			alert('Invalid JSON file or network error.');
		}
	};

	input.click();
}

export function buildCategoryTree(categories: Category[]): CategoryNode[] {
	const categoryMap = new Map<number, CategoryNode>();
	const roots: CategoryNode[] = [];

	// First pass: initialize all categories as nodes with empty children arrays
	for (const cat of categories) {
		categoryMap.set(cat.id, { ...cat, children: [] });
	}

	// Second pass: link them together
	for (const cat of categories) {
		const node = categoryMap.get(cat.id)!;

		if (node.parent_id === null) {
			roots.push(node);
		} else {
			const parent = categoryMap.get(node.parent_id);
			if (parent) {
				parent.children.push(node);
			} else {
				roots.push(node);
			}
		}
	}

	return roots;
}
