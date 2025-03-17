<script lang="ts">
	import type { Snippet } from 'svelte';
	import { MenuIcon } from '../icons';
	import { unfoldHeight } from '$lib/transitions';

	let {
		children,
		collapsed = true,
		ontoggle
	}: {
		children?: Snippet;
		collapsed?: boolean;
		ontoggle?: () => void;
	} = $props();
</script>

<div class="menu-bar">
	{#if !collapsed}
		<nav transition:unfoldHeight>
			<ul>
				{@render children?.()}
			</ul>
		</nav>
	{/if}

	<button class="hamburger" onclick={() => ontoggle?.()}>
		<MenuIcon />
	</button>
</div>

<style>
	.menu-bar {
		display: flex;
		flex-direction: column;
		border-top: 1px solid var(--element-color-700);
		background-color: var(--element-color-900);
	}
	.menu-bar ul {
		list-style: none;
		padding: 0;
		margin: 0;
	}
	.menu-bar .hamburger {
		all: unset;
		margin-left: auto;
		padding: 0.8em;
		font-size: 1em;
		line-height: 0;
		color: var(--element-color-700);
		cursor: pointer;
	}
	.menu-bar .hamburger:hover {
		color: var(--element-color-300);
	}
	.menu-bar .hamburger:active {
		color: var(--element-color-200);
	}
</style>
