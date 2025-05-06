<script lang="ts">
	import { type Snippet } from 'svelte';

	let {
		children,
		style
	}: {
		children?: Snippet;
		style?: string;
	} = $props();
</script>

<div class="fixed" {style}>
	{@render children?.()}
</div>

<div class="fill">
	{@render children?.()}
	<div class="block"></div>
</div>

<style>
	.fixed {
		position: fixed;
	}
	.fill {
		position: relative;
		opacity: 0;
		z-index: -100;
	}
	.fill .block {
		position: absolute;
		inset: 0;
	}
</style>
