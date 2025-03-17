<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<span class="label">
	<span class="icon-align"></span>
	{@render children?.()}
</span>

<style>
	.label {
		background-color: var(--theme-color-primary-700);
		border: 1px solid var(--theme-color-primary-600);
		border-radius: 0.125em;
		padding: 0 0.2em;
		font-size: 0.8em;
		display: inline-flex;
		align-items: center;
		gap: 0.2em;
	}
	.icon-align {
		margin-top: 0.7em;
		margin-right: -0.2em;
	}
</style>
