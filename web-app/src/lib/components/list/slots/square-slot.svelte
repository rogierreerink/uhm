<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<div class="square">
	{@render children?.()}
</div>

<style>
	.square {
		aspect-ratio: 1/1;
		height: 100%;
		display: flex;
	}
</style>
