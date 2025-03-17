<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';

	let {
		children,
		fill = false,
		...anchorProps
	}: {
		children?: Snippet;
		fill?: Boolean;
	} & HTMLAnchorAttributes = $props();
</script>

<a {...anchorProps} class:fill>
	{@render children?.()}
</a>

<style>
	a {
		all: unset;
		flex: 1;
		display: flex;
		position: relative;
		color: var(--theme-color-primary-700);
		cursor: pointer;
	}
	a:hover {
		color: var(--theme-color-primary-300);
	}
	a:active {
		color: var(--theme-color-primary-200);
	}
	a.fill {
		flex: 1;
	}
</style>
