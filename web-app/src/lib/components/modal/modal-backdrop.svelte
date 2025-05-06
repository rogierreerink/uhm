<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children,
		onclose
	}: {
		children?: Snippet;
		onclose?: () => void;
	} = $props();
</script>

<div class="backdrop" onclick={() => onclose?.()} role="presentation">
	{@render children?.()}
</div>

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.4);
		display: flex;
		justify-content: center;
		align-items: center;
	}
</style>
