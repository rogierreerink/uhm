<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		children,
		...buttonProps
	}: {
		children?: Snippet;
	} & HTMLButtonAttributes = $props();
</script>

<button {...buttonProps}>
	{@render children?.()}
</button>

<style>
	button {
		all: unset;
		display: inline-flex;
		gap: 0.25em;
		cursor: pointer;
	}
	button:hover {
		color: var(--theme-color-primary-200);
	}
	button:active {
		color: var(--theme-color-primary-200);
	}
	button:disabled {
		color: inherit;
		cursor: default;
	}
</style>
