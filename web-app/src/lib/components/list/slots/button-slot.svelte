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
		cursor: pointer;
	}
	button:hover {
		color: var(--theme-color-primary-50);
	}
	button:disabled {
		color: var(--theme-color-primary-600);
		cursor: default;
	}
	button.fill {
		flex: 1;
	}
</style>
