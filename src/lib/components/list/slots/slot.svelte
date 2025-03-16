<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children,
		fill = false
	}: {
		children?: Snippet;
		fill?: Boolean;
	} = $props();
</script>

<div class="slot" class:fill>
	{@render children?.()}
</div>

<style>
	.slot {
		display: flex;
	}
	.slot.fill {
		flex: 1;
	}
</style>
