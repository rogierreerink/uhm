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
		color: var(--element-color-400);
		--slot-padding: 0.3em 0.4em;
	}
	li:not(:last-child) {
		border-bottom: 1px solid var(--element-color-700);
	}
</style>
