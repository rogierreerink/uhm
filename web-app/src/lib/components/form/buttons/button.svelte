<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { getGroupContext } from '.';

	let {
		children,
		...buttonProps
	}: {
		children?: Snippet;
	} & HTMLButtonAttributes = $props();

	const groupContext = getGroupContext();
</script>

<button
	{...buttonProps}
	class:horizontal-group={groupContext?.orientation === 'horizontal'}
	class:vertical-group={groupContext?.orientation === 'vertical'}
>
	{@render children?.()}
</button>

<style>
	button {
		all: unset;
		display: flex;
		gap: 0.25em;
		justify-content: center;
		align-items: center;
		padding: 0.3em 0.4em;
		border: 1px solid var(--theme-color-primary-700);
		border-radius: 0.25em;
		box-shadow: 0.25em 0.25em 0 var(--theme-color-primary-950);
		background-color: var(--theme-color-primary-800);
		color: var(--theme-color-primary-200);
		cursor: pointer;
		user-select: none;
		transition: ease 0.1s;
		text-wrap: nowrap;
	}
	button.horizontal-group:not(:last-child) {
		border-top-right-radius: 0;
		border-bottom-right-radius: 0;
	}
	button.horizontal-group:not(:first-child) {
		border-top-left-radius: 0;
		border-bottom-left-radius: 0;
	}
	button.vertical-group:not(:last-child) {
		border-bottom-left-radius: 0;
		border-bottom-right-radius: 0;
	}
	button.vertical-group:not(:first-child) {
		border-top-left-radius: 0;
		border-top-right-radius: 0;
	}
	button:hover {
		color: var(--theme-color-primary-50);
	}
	button:active {
		margin-top: 0.125em;
		margin-bottom: -0.125em;
		box-shadow: 0.125em 0.125em 0 var(--theme-color-primary-950);
	}
	button.vertical-group:active {
		margin: 0;
		margin-left: 0.125em;
		margin-right: -0.125em;
	}
	button:disabled {
		color: var(--theme-color-primary-400);
		margin: 0;
		cursor: default;
	}
</style>
