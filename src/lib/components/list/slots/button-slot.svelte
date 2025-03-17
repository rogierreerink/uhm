<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';

	let {
		children,
		fill = false,
		...buttonProps
	}: {
		children?: Snippet;
		fill?: Boolean;
	} & HTMLButtonAttributes = $props();
</script>

<button {...buttonProps} class:fill>
	{@render children?.()}
</button>

<style>
	button {
		all: unset;
		display: flex;
		position: relative;
		justify-content: center;
		align-items: center;
		color: var(--theme-color-primary-700);
		cursor: pointer;
	}
	button:hover {
		color: var(--theme-color-primary-300);
	}
	button:active {
		color: var(--theme-color-primary-200);
	}
	button.fill {
		flex: 1;
	}
</style>
