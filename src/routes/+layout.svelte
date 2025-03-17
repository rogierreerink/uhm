<script lang="ts">
	import './global.css';
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { MenuBar, MenuItem } from '$lib/components/menu';
	import { FixSpace } from '$lib/components/fix';
	import { afterNavigate, onNavigate } from '$app/navigation';

	let {
		children
	}: {
		children: Snippet;
	} = $props();

	const menu = [
		{ label: 'shopping list', link: '/' },
		{ label: 'products', link: '/products' },
		{ label: 'recipes', link: '/recipes' },
		{ label: 'elements', link: '/elements' }
	];

	let menuCollapsed = $state(true);
	let menuCollapseTimer = $state<number>();
	const menuCollapseAfterMs = 2000;

	onNavigate(() => {
		if (menuCollapseTimer !== undefined) {
			clearTimeout(menuCollapseTimer);
			menuCollapseTimer = undefined;
		}
	});

	afterNavigate(() => {
		if (!menuCollapsed) {
			menuCollapseTimer = setTimeout(() => (menuCollapsed = true), menuCollapseAfterMs);
		}
	});
</script>

<div class="container">
	<div class="page">
		<div class="fill">
			{@render children()}
		</div>
	</div>

	<FixSpace style="bottom: 0; width 100%;">
		<div class="menu">
			<MenuBar collapsed={menuCollapsed} ontoggle={() => (menuCollapsed = !menuCollapsed)}>
				{#each menu as item}
					<MenuItem link={item.link} current={page.url.pathname === item.link}>
						{item.label}
					</MenuItem>
				{/each}
			</MenuBar>
		</div>
	</FixSpace>
</div>

<style>
	.container {
		display: grid;
		grid-template-rows: 1fr auto;
		grid-template-areas:
			'page'
			'menu';
		min-height: 100vh;
	}
	.container .page {
		grid-area: page;
		padding: 1em;
	}
	.container .page .fill {
		width: 100%;
		max-width: 800px;
		margin: 0 auto;
	}
	.container .menu {
		grid-area: menu;
		width: 100vw;
		z-index: 100;
	}
</style>
