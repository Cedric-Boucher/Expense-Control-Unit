<script lang="ts">
    import { goto } from '$app/navigation';
    import { check_login, login } from '$lib/api';
    import { auth } from '$lib/stores/auth';
    import type { NewUser } from '$lib/types';
    import { onMount } from 'svelte';

    let username = '';
    let password = '';
    let error = '';

    onMount(async () => {
        await check_login();
        if ($auth.isLoggedIn) {
            goto('/transactions');
        }
    });

    async function submit() {
        error = '';

        if (!username || !password) {
            error = 'Username and password are required.';
            return;
        }

        const payload: NewUser = {
            username,
            password,
        };

        try {
            await login(payload);
            goto('/transactions');
        } catch (e) {
            error = 'Invalid credentials';
            console.error(e);
        }
    }
</script>

<h1>Login</h1>
<form on:submit|preventDefault={submit} class="space-y-4 max-w-sm">
    <input bind:value={username} placeholder="Username" class="w-full p-2 rounded border" />
    <input bind:value={password} type="password" placeholder="Password" class="w-full p-2 rounded border" />
    <button type="submit" class="w-full p-2 bg-blue-600 text-white rounded">Login</button>
    {#if error}<p class="text-red-500">{error}</p>{/if}
</form>
