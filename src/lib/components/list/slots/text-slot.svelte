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

<div class="text" class:fill>
	{@render children?.()}
</div>

<style>
	.text {
		all: unset;
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding: var(--slot-padding);
		text-wrap: nowrap;
	}
	.text.fill {
		flex: 1;
	}
</style>
