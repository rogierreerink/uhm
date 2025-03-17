<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Border } from '../boxes';

	let {
		children,
		footer
	}: {
		children?: Snippet;
		footer?: Snippet;
	} = $props();
</script>

<div class="modal">
	<Border>
		{@render children?.()}
	</Border>

	<div class="footer">
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
</style>
