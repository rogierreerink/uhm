<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<li>
	{@render children?.()}
</li>

<style>
	li {
		display: flex;
		color: var(--theme-color-primary-400);
		--slot-padding: 0.3em 0.4em;
	}
	li:not(:last-child) {
		border-bottom: 1px solid var(--theme-color-primary-700);
	}
</style>
