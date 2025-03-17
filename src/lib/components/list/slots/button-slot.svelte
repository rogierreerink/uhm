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
		color: var(--element-color-700);
		cursor: pointer;
	}
	button:hover {
		color: var(--element-color-300);
	}
	button:active {
		color: var(--element-color-200);
	}
	button.fill {
		flex: 1;
	}
</style>
