<script lang="ts">
	import { ArrowIcon } from '../icons';
	import { ListItem } from '../list';
	import { AnchorSlot, TextSlot, IconSlot } from '../list/slots';
	import type { Snippet } from 'svelte';

	let {
		children,
		link,
		current
	}: {
		children?: Snippet;
		link?: string;
		current?: boolean;
	} = $props();
</script>

<li>
	<a href={link} class:current>
		{@render children?.()}
	</a>
</li>

<style>
	li {
		margin: 0.6em 1em;
	}
	li a {
		all: unset;
		display: block;
		cursor: pointer;
		color: var(--color-zinc-700);
	}
	li a.current {
		color: var(--color-zinc-400);
	}
</style>
