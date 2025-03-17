<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<div class="border">
	{@render children?.()}
</div>

<style>
	.border {
		border: 1px solid var(--theme-color-primary-700);
		border-radius: 0.25em;
		box-shadow: 0.25em 0.25em 0 var(--theme-color-primary-950);
		background-color: var(--theme-color-primary-900);
	}
</style>
