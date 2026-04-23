import { format } from 'date-fns';
import { getTransactions, getCategories, uploadUserData } from '$lib/api';
import type { Category, CategoryNode, Transaction } from '$lib/types';
import { invalidateAll } from '$app/navigation';

export function formatTimestampLocal(isoString: string): string {
    const date = new Date(isoString); // converts to local time
    return format(date, "yyyy-MM-dd'T'HH:mm:ss");
}

export function formatTimestampLocalForDisplay(isoString: string): string {
    const date = new Date(isoString); // converts to local time
    return format(date, "yyyy-MM-dd HH:mm:ss");
}

export async function exportUserDataToFile() {
    try {
        const [transactions, categories]: [Transaction[], Category[]] = await Promise.all([
            getTransactions(),
            getCategories()
        ]);

        const roots = buildCategoryTree(categories);

        const categoryIdToName = new Map(categories.map(c => [c.id, c.name]));

        // Flatten the tree topologically (parents first, then children)
        const sortedExportCategories = [];
        const queue = [...roots];

        while (queue.length > 0) {
            const current = queue.shift()!;

            // Extract what we need and map parent_id to parent_name
            const { id, parent_id, children, ...categoryData } = current;

            sortedExportCategories.push({
                ...categoryData,
                parent_name: parent_id ? categoryIdToName.get(parent_id) || null : null
            });

            // Queue up the children for the next passes
            if (children && children.length > 0) {
                queue.push(...children);
            }
        }

        const exportData = {
            transactions,
            categories: sortedExportCategories
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
            await uploadUserData(text);
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
