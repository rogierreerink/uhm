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
		display: flex;
		position: relative;
		color: var(--element-color-400);
		cursor: pointer;
	}
	a:hover {
		color: var(--element-color-300);
	}
	a:active {
		color: var(--element-color-200);
	}
	a.fill {
		flex: 1;
	}
</style>
