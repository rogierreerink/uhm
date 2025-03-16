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
		color: var(--slot-color, var(--color-zinc-700));
		cursor: pointer;
	}
	a:hover {
		color: var(--slot-color-hover, var(--color-zinc-300));
	}
	a:active {
		color: var(--slot-color-active, var(--color-zinc-200));
	}
	a.fill {
		flex: 1;
	}
</style>
