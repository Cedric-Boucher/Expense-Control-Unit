import { goto } from "$app/navigation";
import { auth } from "./stores/auth";
import type { Transaction, NewTransaction, NewUser, User } from "./types";

const API_BASE = 'http://localhost:8000';

export async function getTransactions(): Promise<Transaction[]> {
    const res = await fetch(`${API_BASE}/api/transactions`, {
        credentials: 'include'
    });
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
        body: JSON.stringify(payload),
        credentials: 'include',
    });

    if (!res.ok) {
        throw new Error('Failed to create transaction');
    }

    return await res.json();
}

export async function login(payload: NewUser): Promise<void> {
    const res = await fetch(`${API_BASE}/api/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        credentials: 'include',
    });

    if (res.ok) {
        auth.set({ isLoggedIn: true });
    } else {
        throw new Error('Invalid credentials');
    }
}

export async function signup(payload: NewUser): Promise<void> {
    const res = await fetch(`${API_BASE}/api/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        credentials: 'include',
    });


    if (res.ok) {
        auth.set({ isLoggedIn: true });
    } else {
        throw new Error((await res.text()).valueOf());
    }
}

export async function logout(): Promise<void> {
    await fetch(`${API_BASE}/api/logout`, {
        method: 'POST',
        credentials: 'include',
    });
    auth.set({ isLoggedIn: false });
    goto('/login');
}

export async function check_login(): Promise<void> {
    const res = await fetch(`${API_BASE}/api/me`, {
        credentials: 'include'
    });
    auth.set({ isLoggedIn: res.ok });
}

export async function load_user(): Promise<{user: User | null}> {
    const res = await fetch(`${API_BASE}/api/me`, {
        credentials: 'include'
    });
    if (res.ok) {
        const user = await res.json();
        return { user };
    } else {
        return { user: null };
    }
};
