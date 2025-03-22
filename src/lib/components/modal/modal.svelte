<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Box } from '../boxes';

	let {
		children,
		footer,
		size = 'full'
	}: {
		children?: Snippet;
		footer?: Snippet;
		size?: 'full' | 'small';
	} = $props();

	const stopClickPropagation = {
		onclick: (e: MouseEvent) => e.stopPropagation()
	};
</script>

<div class="modal" class:full={size === 'full'} class:small={size === 'small'}>
	<Box>
		<div class="main" {...stopClickPropagation}>
			{@render children?.()}
		</div>
	</Box>

	<div class="footer" {...stopClickPropagation}>
		{@render footer?.()}
	</div>
</div>

<style>
	.modal {
		display: flex;
		flex-direction: column;
		gap: 1em;
		margin: 1em;
		max-width: 800px;
	}
	.modal.small {
		width: 100%;
		max-width: 400px;
	}
	.modal .footer {
		display: flex;
		justify-content: end;
	}
</style>
