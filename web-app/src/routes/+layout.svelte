<script lang="ts">
	import './global.css';
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { MenuBar, MenuItem } from '$lib/components/menu';
	import { FixSpace } from '$lib/components/fix';
	import { afterNavigate, onNavigate } from '$app/navigation';
	import { type GetResponse as Lists } from '$lib/data/lists/collection';

	let {
		children,
		data
	}: {
		children: Snippet;
		data: { lists: Lists };
	} = $props();

	let menu = $derived([
		...data.lists.data.map((list) => ({
			label: list.data.name.toLocaleLowerCase(),
			link: `/lists/${list.id}`
		})),
		{ label: 'products', link: '/products' },
		{ label: 'recipes', link: '/recipes' },
		{ label: 'elements', link: '/elements' }
	]);

	let menuCollapsed = $state(true);
	let menuCollapseTimer = $state<number>();
	const menuCollapseAfterMs = 2000;

	function startMenuCollapseTimer() {
		if (!menuCollapsed) {
			menuCollapseTimer = setTimeout(() => (menuCollapsed = true), menuCollapseAfterMs);
		}
	}

	function clearMenuCollapseTimer() {
		if (menuCollapseTimer !== undefined) {
			clearTimeout(menuCollapseTimer);
			menuCollapseTimer = undefined;
		}
	}
</script>

<div class="container">
	<div class="page">
		<div class="fill">
			{@render children()}
		</div>
	</div>

	<FixSpace style="inset: 0; top: auto;">
		<div
			class="menu"
			onmouseenter={() => clearMenuCollapseTimer()}
			onmouseleave={() => startMenuCollapseTimer()}
			role="presentation"
		>
			<MenuBar
				collapsed={menuCollapsed}
				ontoggle={() => {
					menuCollapsed = !menuCollapsed;
					clearMenuCollapseTimer();
				}}
			>
				{#each menu as item}
					<MenuItem
						link={item.link}
						current={(item.link.length > 1 && page.url.pathname.startsWith(item.link)) ||
							page.url.pathname === item.link}
					>
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
		width: 100%;
		z-index: 1000;
	}
</style>
