import { goto } from '$app/navigation';
import { resolve } from '$app/paths';
import { auth } from './stores/auth.svelte';
import type {
	Transaction,
	NewTransaction,
	NewUser,
	User,
	Category,
	NewCategory,
	ImportPayload,
	Tag,
	NewTag
} from './types';

const API_BASE = '/api';

export async function getTransactions(): Promise<Transaction[]> {
	const res = await fetch(`${API_BASE}/transactions`, {
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

	const res = await fetch(`${API_BASE}/transactions`, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(payload),
		credentials: 'include'
	});

	if (!res.ok) {
		throw new Error('Failed to create transaction');
	}

	return await res.json();
}

export async function login(payload: NewUser): Promise<void> {
	const res = await fetch(`${API_BASE}/login`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(payload),
		credentials: 'include'
	});

	if (res.ok) {
		auth.isLoggedIn = true;
	} else {
		throw new Error('Invalid credentials');
	}
}

export async function signup(payload: NewUser): Promise<void> {
	const res = await fetch(`${API_BASE}/signup`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(payload),
		credentials: 'include'
	});

	if (res.ok) {
		auth.isLoggedIn = true;
	} else {
		throw new Error((await res.text()).valueOf());
	}
}

export async function logout(): Promise<void> {
	await fetch(`${API_BASE}/logout`, {
		method: 'POST',
		credentials: 'include'
	});
	auth.isLoggedIn = false;
	goto(resolve('/login'));
}

export async function check_login(): Promise<void> {
	const res = await fetch(`${API_BASE}/me`, {
		credentials: 'include'
	});
	auth.isLoggedIn = res.ok;
}

export async function load_user(): Promise<{ user: User | null }> {
	const res = await fetch(`${API_BASE}/me`, {
		credentials: 'include'
	});
	if (res.ok) {
		const user = await res.json();
		return { user };
	} else {
		return { user: null };
	}
}

export async function getCategories(): Promise<Category[]> {
	const res = await fetch(`${API_BASE}/categories`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to fetch categories');
	return await res.json();
}

export async function createCategory(payload: NewCategory): Promise<Category> {
	const res = await fetch(`${API_BASE}/categories`, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(payload),
		credentials: 'include'
	});

	if (!res.ok) {
		throw new Error('Failed to create category');
	}

	return await res.json();
}

export async function getTransaction(id: string): Promise<Transaction> {
	const res = await fetch(`${API_BASE}/transactions/${id}`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to fetch transaction');
	return await res.json();
}

export async function updateTransaction(id: string, data: NewTransaction) {
	const res = await fetch(`${API_BASE}/transactions/${id}`, {
		method: 'PUT',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(data),
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to update transaction');
}

export async function deleteTransaction(id: string) {
	const res = await fetch(`${API_BASE}/transactions/${id}`, {
		method: 'DELETE',
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to delete transaction');
}

export async function getCategory(id: string): Promise<Category> {
	const res = await fetch(`${API_BASE}/categories/${id}`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to fetch category');
	return await res.json();
}

export async function updateCategory(id: string, data: NewCategory) {
	const res = await fetch(`${API_BASE}/categories/${id}`, {
		method: 'PUT',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(data),
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to update category');
}

export async function deleteCategory(id: string) {
	const res = await fetch(`${API_BASE}/categories/${id}`, {
		method: 'DELETE',
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to delete category');
}

export async function getCategoryTransactions(category_id: string): Promise<Transaction[]> {
	const res = await fetch(`${API_BASE}/categories/${category_id}/transactions`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to fetch transactions');
	return await res.json();
}

export async function getTags(): Promise<Tag[]> {
	const res = await fetch(`${API_BASE}/tags`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to fetch tags');
	return await res.json();
}

export async function createTag(payload: NewTag): Promise<Tag> {
	const res = await fetch(`${API_BASE}/tags`, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(payload),
		credentials: 'include'
	});

	if (!res.ok) {
		if (res.status === 409) {
			throw new Error('A tag with this name already exists.');
		}
		throw new Error('Failed to create tag');
	}

	return await res.json();
}

export async function getTag(id: string): Promise<Tag> {
	const res = await fetch(`${API_BASE}/tags/${id}`, {
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to fetch tag');
	return await res.json();
}

export async function updateTag(id: string, data: NewTag) {
	const res = await fetch(`${API_BASE}/tags/${id}`, {
		method: 'PUT',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(data),
		credentials: 'include'
	});

	if (!res.ok) {
		if (res.status === 409) {
			throw new Error('A tag with this name already exists.');
		}
		throw new Error('Failed to update tag');
	}
}

export async function deleteTag(id: string) {
	const res = await fetch(`${API_BASE}/tags/${id}`, {
		method: 'DELETE',
		credentials: 'include'
	});
	if (!res.ok) throw new Error('Failed to delete tag');
}

export async function uploadUserData(payload: ImportPayload) {
	const res = await fetch(`${API_BASE}/import`, {
		method: 'POST',
		headers: {
			'content-type': 'application/json'
		},
		body: JSON.stringify(payload),
		credentials: 'include'
	});

	if (res.ok) {
		alert('Data imported successfully!');
	} else {
		const error = await res.text();
		alert(`Import failed: ${error}`);
	}
}
