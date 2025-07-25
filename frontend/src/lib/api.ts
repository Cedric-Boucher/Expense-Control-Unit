import type { Transaction, NewTransaction } from "./types";

const API_BASE = 'http://localhost:8000';

export async function getTransactions(): Promise<Transaction[]> {
    const res = await fetch(`${API_BASE}/api/transactions`);
    if (!res.ok) throw new Error('Failed to fetch transactions');
    return await res.json();
}

export async function createTransaction(payload: NewTransaction): Promise<Transaction> {
    // Convert timestamp to ISO format (UTC) if present
    if (payload.created_at) {
        payload.created_at = new Date(payload.created_at).toISOString();
    }

    const res = await fetch(`${API_BASE}/api/transactions`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(payload)
    });

    if (!res.ok) {
        throw new Error('Failed to create transaction');
    }

    return await res.json();
}
