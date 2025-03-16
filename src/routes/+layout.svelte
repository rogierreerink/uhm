<script lang="ts">
	import './global.css';
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { Menu, MenuItem } from '$lib/components/menu';
	import { FixWidth } from '$lib/components/fix';

	let {
		children
	}: {
		children: Snippet;
	} = $props();

	const menu = [
		{ label: 'cart	', link: '/' },
		{ label: 'products', link: '/products' },
		{ label: 'recipes', link: '/recipes' },
		{ label: 'elements', link: '/elements' }
	];
</script>

<div class="container">
	<div class="menu">
		<FixWidth>
			<Menu>
				{#each menu as item}
					<MenuItem link={item.link} current={page.url.pathname === item.link}>
						{item.label}
					</MenuItem>
				{/each}
			</Menu>
		</FixWidth>
	</div>

	<div class="page">
		{@render children()}
	</div>
</div>

<style>
	.container {
		display: flex;
		height: 100vh;
		margin: 1em;
		gap: 1em;
	}
	.container .page {
		flex: 1;
	}
</style>
