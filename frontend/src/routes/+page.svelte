<script lang="ts">
	import { onMount } from "svelte";

    let healthMessage = 'Loading...';

    onMount(async () => {
        try {
            const res = await fetch('http://localhost:8000/api/health');
            healthMessage = await res.text();
        } catch (e) {
            healthMessage = 'Failed to connect to backend';
        }
    });
</script>

<h1>My Finance Tracker</h1>
<p>Backend status: {healthMessage}</p>
