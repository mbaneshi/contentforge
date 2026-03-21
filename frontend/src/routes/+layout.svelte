<script lang="ts">
	import '../app.css';
	import Nav from '$lib/components/Nav.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();
</script>

<div class="flex min-h-screen">
	<Nav />
	<main class="ml-60 flex-1 p-6">
		{@render children()}
	</main>
</div>
