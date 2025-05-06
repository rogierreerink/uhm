<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLLabelAttributes } from 'svelte/elements';

	let {
		children,
		...props
	}: {
		children?: Snippet;
	} & HTMLLabelAttributes = $props();
</script>

<label class="label" {...props}>
	{@render children?.()}
</label>

<style>
	.label {
		display: block;
		padding: 0.3em 0.4em;
		font-style: italic;
	}
</style>
