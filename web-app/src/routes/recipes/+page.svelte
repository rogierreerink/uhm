<script lang="ts">
	import { Box } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import { TextAnchorSlot } from '$lib/components/list/slots';
	import type { GetResponse } from '$lib/data/pages/collection';
	import type { GetResponse as GetListsResponse } from '$lib/data/lists/collection';

	let {
		data
	}: {
		data: {
			recipes: GetResponse;
		};
	} = $props();
</script>

<svelte:head>
	<title>Recipes</title>
</svelte:head>

<section class="page">
	<h1>Recipes</h1>

	<Box>
		<List>
			{#each data.recipes.data as recipe (recipe.id)}
				<ListItem>
					<TextAnchorSlot fill href={`/recipes/${recipe.id}`}>
						{recipe.data.name}
					</TextAnchorSlot>
				</ListItem>
			{/each}
		</List>
	</Box>
</section>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
</style>
