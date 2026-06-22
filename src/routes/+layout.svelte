<script lang="ts">
	import './layout.css';
	import './regenforge.css';
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<div class="min-h-screen bg-surface-950-50 text-white">
	{@render children()}

</div>
