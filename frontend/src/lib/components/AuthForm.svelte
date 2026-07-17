<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { login, signup } from '$lib/api';
	import type { NewUser } from '$lib/types';
	import AsyncButton from '$lib/components/AsyncButton.svelte';

	let { mode }: { mode: 'login' | 'signup' } = $props();

	let username = $state('');
	let password = $state('');
	let error = $state('');

	let isLogin = $derived(mode === 'login');
	let title = $derived(isLogin ? 'Login' : 'Sign Up');
	let buttonColor = $derived(isLogin ? 'bg-blue-600' : 'bg-green-600');

	async function submit() {
		error = '';

		const payload: NewUser = {
			username,
			password
		};

		try {
			if (isLogin) {
				await login(payload);
			} else {
				await signup(payload);
			}
			await goto(resolve('/transactions'));
		} catch (e) {
			if (isLogin) {
				error = 'Invalid credentials';
			} else {
				const err = e as Error;
				error = err.message || 'An error occurred during sign up.';
			}
			console.error(e);
		}
	}
</script>

<h1>{title}</h1>
<form onsubmit={(e) => e.preventDefault()} class="space-y-4 max-w-sm">
	<input
		bind:value={username}
		placeholder="Username"
		required
		class="w-full p-2 rounded border"
	/>
	<input
		bind:value={password}
		type="password"
		placeholder="Password"
		required
		class="w-full p-2 rounded border"
	/>
	<AsyncButton type="submit" action={submit} class="w-full p-2 {buttonColor} text-white rounded">
		{title}
	</AsyncButton>
	{#if error}<p class="text-red-500">{error}</p>{/if}
</form>
