<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { login, signup } from '$lib/api';
	import type { NewUser } from '$lib/types';

	let { mode }: { mode: 'login' | 'signup' } = $props();

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let isSubmitting = $state(false);

	let isLogin = $derived(mode === 'login');
	let title = $derived(isLogin ? 'Login' : 'Sign Up');
	let buttonColor = $derived(isLogin ? 'bg-blue-600' : 'bg-green-600');

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		error = '';
		isSubmitting = true;

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
			goto(resolve('/transactions'));
		} catch (e) {
			if (isLogin) {
				error = 'Invalid credentials';
			} else {
				const err = e as Error;
				error = err.message || 'An error occurred during sign up.';
			}
			console.error(e);
			isSubmitting = false;
		}
	}
</script>

<h1>{title}</h1>
<form onsubmit={submit} class="space-y-4 max-w-sm">
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
	<button
		type="submit"
		disabled={isSubmitting}
		class="w-full p-2 {buttonColor} text-white rounded disabled:opacity-50 disabled:cursor-not-allowed"
	>
		{isSubmitting ? 'Processing...' : title}
	</button>
	{#if error}<p class="text-red-500">{error}</p>{/if}
</form>
