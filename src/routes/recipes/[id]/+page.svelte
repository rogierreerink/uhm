<script lang="ts">
	import { IngredientCollectionBlock, MarkdownBlock } from '$lib/components/blocks';
	import { Breadcrumb, BreadcrumbTrail } from '$lib/components/breadcrumb';
	import recipe, { type GetResponse } from '$lib/data/pages/resource';
	import { type GetResponse as GetListsResponse } from '$lib/data/lists/collection';
	import { invalidate } from '$app/navigation';

	let {
		data
	}: {
		data: {
			recipe: GetResponse;
			lists: GetListsResponse;
		};
	} = $props();

	async function invalidatePage() {
		await invalidate(recipe.url(data.recipe.id));
	}
</script>

<svelte:head>
	<title>{data.recipe.data.name}</title>
</svelte:head>

<section class="page">
	<h1>{data.recipe.data.name}</h1>

	<BreadcrumbTrail>
		<Breadcrumb href=".">recipes</Breadcrumb>
		<Breadcrumb>{data.recipe.data.name.toLocaleLowerCase()}</Breadcrumb>
	</BreadcrumbTrail>

	{#each data.recipe.data.blocks as block, idx (idx)}
		{@const kind = block.data.kind}

		{#if kind.type === 'markdown'}
			<MarkdownBlock data={kind.data} />
		{:else if kind.type === 'ingredient_collection'}
			<IngredientCollectionBlock
				collection={kind}
				lists={data.lists.data}
				oninvalidate={invalidatePage}
			/>
		{/if}
	{/each}
</section>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
</style>
