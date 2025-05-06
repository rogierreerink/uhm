<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<div class="icon">
	{@render children?.()}
</div>

<style>
	.icon {
		display: flex;
		justify-content: center;
		align-items: center;
		padding: var(--slot-padding);
	}
</style>
