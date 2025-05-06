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
		border-top: 1px solid var(--theme-color-primary-700);
		background-color: var(--theme-color-primary-900);
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
		color: var(--theme-color-primary-200);
		cursor: pointer;
	}
	.menu-bar .hamburger:hover {
		color: var(--theme-color-primary-50);
	}
</style>
