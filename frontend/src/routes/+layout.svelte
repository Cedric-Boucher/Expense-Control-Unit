<script lang="ts">
    import { auth } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import '../app.css';
    import { onMount, type Snippet } from 'svelte';
    import { check_login, logout } from '$lib/api';
    import type { LayoutData } from './$types';
    import { page } from '$app/state';

    let { data, children }: {data: LayoutData, children: Snippet } = $props()

    const currentPath = $derived(page.url.pathname);

    let darkMode = $state(false);

    onMount(check_login);

    onMount(() => {
        auth.set({ isLoggedIn: !!data.user });
    });

    onMount(() => {
        const stored = localStorage.getItem('theme');
        if (stored === 'dark') {
            darkMode = true;
            document.documentElement.classList.add('dark');
        } else {
            darkMode = false;
            document.documentElement.classList.remove('dark');
        }
    });

    function toggleDarkMode() {
        darkMode = !darkMode;
        document.documentElement.classList.toggle('dark', darkMode);
        localStorage.setItem('theme', darkMode ? 'dark' : 'light');
    }

    function navButtonClasses(pathPrefix: string): string {
        const base = 'px-3 py-2 rounded font-medium';
        const active = 'bg-blue-100 dark:bg-gray-700 text-blue-900 dark:text-white';
        const inactive = 'hover:bg-blue-100 dark:hover:bg-gray-700 text-blue-700 dark:text-blue-300';

        return `${base} ${currentPath.startsWith(pathPrefix) ? active : inactive}`;
    }
</script>

<nav class="bg-white dark:bg-gray-800 shadow sticky top-0 z-10">
    <div class="max-w-4xl mx-auto px-4 py-3 flex justify-between items-center">
        <div class="flex gap-4">
            <button onclick={() => goto('/transactions')} class={navButtonClasses('/transactions')}>
                Transactions
            </button>
            <button onclick={() => goto('/transactions/new')} class={navButtonClasses('/transactions/new')}>
                New Transaction
            </button>
            <button onclick={() => goto('/categories')} class={navButtonClasses('/categories')}>
                Categories
            </button>
            <button onclick={() => goto('/categories/new')} class={navButtonClasses('/categories/new')}>
                New Category
            </button>
        </div>

        <div class="flex gap-4">
            {#if $auth.isLoggedIn}
                <button onclick={logout} class="px-3 py-2 rounded hover:bg-blue-100 dark:hover:bg-gray-700 text-red-500 dark:text-red-500 font-medium">
                    Logout
                </button>
            {:else}
                <button class="px-3 py-2 rounded hover:bg-blue-100 dark:hover:bg-gray-700 text-blue-700 dark:text-blue-300 font-medium" onclick={() => goto('/login')}>
                    Login
                </button>
                <button class="px-3 py-2 rounded hover:bg-blue-100 dark:hover:bg-gray-700 text-blue-700 dark:text-blue-300 font-medium" onclick={() => goto('/signup')}>
                    Sign Up
                </button>
            {/if}
            <button onclick={toggleDarkMode} class="px-3 py-2 rounded text-sm bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-900 dark:text-gray-100">
                {darkMode ? '🌙 Dark' : '☀️ Light'}
            </button>
        </div>
    </div>
</nav>

<main class="max-w-4xl mx-auto px-4 py-6">
    {@render children()}
</main>
