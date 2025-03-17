<script lang="ts">
	import type { Snippet } from 'svelte';
	import Box from './box.svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<Box --box-background-color="var(--element-color-800)">
	{@render children?.()}
</Box>
