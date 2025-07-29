import { writable } from 'svelte/store';

export const auth = writable<{ isLoggedIn: boolean }>({ isLoggedIn: false });
