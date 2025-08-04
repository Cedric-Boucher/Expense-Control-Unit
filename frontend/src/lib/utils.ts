import { format } from 'date-fns';
import { getTransactions, getCategories, uploadUserData } from '$lib/api';
import type { Category, Transaction } from '$lib/types';
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

        const exportData = {
            transactions,
            categories
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
