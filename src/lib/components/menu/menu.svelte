<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Border } from '$lib/components/boxes';
	import { List } from '$lib/components/list';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<nav>
	<Border>
		<List>
			{@render children?.()}
		</List>
	</Border>
</nav>

<style>
</style>
