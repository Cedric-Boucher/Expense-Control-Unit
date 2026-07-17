<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	// Extend standard button attributes for strict type safety
	interface Props extends HTMLButtonAttributes {
		// Explicitly require a function that returns a Promise
		action: (event: MouseEvent) => Promise<unknown>;
		// Svelte 5 uses snippets instead of slots
		children: Snippet;
	}

	let {
		action,
		children,
		class: className = '',
		type = 'button', // Default to 'button' to prevent accidental form submissions
		...rest
	}: Props = $props();

	// Svelte 5 reactive state rune
	let isLoading = $state(false);

	async function handleAsyncClick(event: MouseEvent) {
		// Fault tolerance: prevent double-execution if the UI hasn't painted the disabled state yet
		if (isLoading) return;

		isLoading = true;
		try {
			await action(event);
		} catch (error) {
			console.error('AsyncButton action failed:', error);
			// In a larger app, you might dispatch a toast notification event here
		} finally {
			isLoading = false;
		}
	}
</script>

<button
	{type}
	class="async-button {className}"
	disabled={isLoading || rest.disabled}
	aria-busy={isLoading}
	onclick={handleAsyncClick}
	{...rest}
>
	{#if isLoading}
		<span class="spinner" aria-hidden="true">
			<!-- Replace with your preferred SVG icon -->
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="12" y1="2" x2="12" y2="6"></line>
				<line x1="12" y1="18" x2="12" y2="22"></line>
				<line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
				<line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
				<line x1="2" y1="12" x2="6" y2="12"></line>
				<line x1="18" y1="12" x2="22" y2="12"></line>
				<line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
				<line x1="16.24" y1="4.93" x2="19.07" y2="7.76"></line>
			</svg>
		</span>
		<!-- Screen reader text so blind users know what is happening -->
		<span class="sr-only">Processing...</span>
	{:else}
		<!-- Svelte 5 syntax for rendering the children snippet -->
		{@render children()}
	{/if}
</button>

<style>
	.async-button:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	.spinner {
		display: inline-block;
		margin-right: 0.5rem;
		width: 1em;
		height: 1em;
		animation: spin 1s linear infinite;
	}
	.spinner svg {
		width: 100%;
		height: 100%;
	}
	/* Utility class for accessibility */
	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border-width: 0;
	}
	@keyframes spin {
		100% {
			transform: rotate(360deg);
		}
	}
</style>
