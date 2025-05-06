<script lang="ts">
	import type { Snippet } from 'svelte';
	import { setGroupContext } from '.';

	let {
		children,
		orientation = 'horizontal'
	}: {
		children?: Snippet;
		orientation?: 'horizontal' | 'vertical';
	} = $props();

	setGroupContext({
		orientation
	});
</script>

<div class="group" class:vertical={orientation === 'vertical'}>
	{@render children?.()}
</div>

<style>
	.group {
		display: flex;
		gap: 1px;
	}
	.vertical {
		flex-direction: column;
	}
</style>
