<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ChevronIcon } from '../icons';

	let {
		children,
		href
	}: {
		children?: Snippet;
		href?: string;
	} = $props();
</script>

<li class="breadcrumb">
	{#if href}
		<a class="label" {href}>
			{@render children?.()}
		</a>
	{:else}
		<span class="label">
			{@render children?.()}
		</span>
	{/if}
	<div class="icon">
		<ChevronIcon />
	</div>
</li>

<style>
	.breadcrumb {
		display: flex;
		gap: 0.5em;
		color: var(--theme-color-primary-200);
	}
	.breadcrumb a.label {
		all: unset;
		text-decoration: none;
		cursor: pointer;
	}
	.breadcrumb a.label:hover {
		color: var(--theme-color-primary-200);
	}
	.breadcrumb a.label:active {
		color: var(--theme-color-primary-200);
	}
	.breadcrumb .icon {
		display: flex;
		align-items: center;
	}
	.breadcrumb:last-child .icon {
		display: none;
	}
</style>
