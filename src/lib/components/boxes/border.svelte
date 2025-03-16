<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<div class="box">
	{@render children?.()}
</div>

<style>
	.box {
		border: 1px solid var(--box-border-color, var(--color-zinc-700));
		border-radius: 0.25em;
		box-shadow: 0.25em 0.25em 0 var(--box-shadow-color, var(--color-zinc-950));
		background-color: var(--box-bg-color, var(--color-zinc-900));
	}
</style>
