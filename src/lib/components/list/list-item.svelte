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
		color: var(--list-item-color, var(--color-zinc-400));
		--slot-padding: 0.3em 0.4em;
	}
	li:not(:last-child) {
		border-bottom: 1px solid var(--color-zinc-700);
	}
</style>
