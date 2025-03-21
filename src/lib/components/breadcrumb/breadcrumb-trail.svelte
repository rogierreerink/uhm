<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<ul class="breadcrumb-trail">
	{@render children?.()}
</ul>

<style>
	.breadcrumb-trail {
		list-style: none;
		font-size: 0.8em;
		padding: 0.3em 0.4em;
		margin: 0;
		display: flex;
		gap: 0.5em;
	}
</style>
