const API_BASE = 'http://localhost:8000';

export async function getTransactions() {
    const res = await fetch('${API_BASE}/transactions');
    return await res.json();
}
