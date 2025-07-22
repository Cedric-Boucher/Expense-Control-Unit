import type { Transaction } from "./types";

const API_BASE = 'http://localhost:8000';

export async function getTransactions(): Promise<Transaction[]> {
    const res = await fetch(`${API_BASE}/api/transactions`);
    if (!res.ok) throw new Error('Failed to fetch transactions');
    return await res.json();
}
