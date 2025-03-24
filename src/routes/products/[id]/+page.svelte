<script lang="ts">
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import Box from '$lib/components/boxes/box.svelte';
	import { Breadcrumb, BreadcrumbTrail } from '$lib/components/breadcrumb';
	import { Label, TextInput } from '$lib/components/form';
	import { Button, ButtonGroup, InlineButton } from '$lib/components/form/buttons';
	import { CheckIcon, DeleteIcon, UndoIcon } from '$lib/components/icons';
	import { Modal, ModalBackdrop } from '$lib/components/modal';
	import product, { type GetResponse } from '$lib/data/products/resource';
	import products from '$lib/data/products/collection';
	import shoppingList from '$lib/data/shopping-list/collection';
	import shoppingListItem from '$lib/data/shopping-list/resource';

	let {
		data
	}: {
		data: GetResponse;
	} = $props();

	let name = $state(data.data.name);
	let hasChanged = $derived(name !== data.data.name);
	let confirmDeleteModal = $state(false);

	async function saveChanges() {
		await product.patch(page.params.id, { name });
		await invalidate(products.url());
		await invalidate(product.url(page.params.id));
		await invalidate(shoppingList.url());
	}

	async function revertChanges() {
		name = data.data.name;
	}

	async function deleteProduct() {
		await product.delete(page.params.id);
		goto('.', { replaceState: true });
	}

	async function addShoppingListItem() {
		await shoppingList.post({
			data: [{ source: { type: 'product', id: data.id } }]
		});
		await invalidate(products.url());
		await invalidate(product.url(data.id));
		await invalidate(shoppingList.url());
	}

	async function unlinkShoppingListItems() {
		const response = await product.get(data.id);
		if (!response.ok) {
			return;
		}

		for (const { id } of response.data.data.shoppingListItemLinks) {
			await shoppingListItem.patch(id, {
				source: {
					type: 'temporary',
					data: {
						name: response.data.data.name
					}
				}
			});
		}

		await invalidate(products.url());
		await invalidate(product.url(data.id));
		await invalidate(shoppingList.url());
	}

	async function deleteShoppingListItems() {
		for (const { id } of data.data.shoppingListItemLinks) {
			await shoppingListItem.delete(id);
			await invalidate(shoppingListItem.url(id));
		}
		await invalidate(products.url());
		await invalidate(product.url(data.id));
		await invalidate(shoppingList.url());
	}
</script>

<svelte:head>
	<title>Products: {data.data.name.toLocaleLowerCase()}</title>
</svelte:head>

<section class="page">
	<h1>{data.data.name}</h1>

	<BreadcrumbTrail>
		<Breadcrumb href=".">products</Breadcrumb>
		<Breadcrumb>{data.data.name.toLocaleLowerCase()}</Breadcrumb>
	</BreadcrumbTrail>

	<div class="data">
		<div class="property">
			<Label for="name">Name</Label>
			<div class="input">
				<Box>
					<div class="flex">
						<TextInput
							id="name"
							placeholder={data.data.name}
							value={name}
							oninput={(e) => (name = e.currentTarget.value.trim())}
						/>
					</div>
				</Box>
			</div>
		</div>
	</div>

	<div class="statistics">
		{#if data.data.shoppingListItemLinks.length > 0}
			{@const link_count = data.data.shoppingListItemLinks.length}
			<div>
				{#if new Intl.PluralRules('en-US').select(link_count) === 'one'}
					<a href={`/?product-highlight=${data.id}`}>{link_count} reference to shopping list</a>
					<InlineButton onclick={() => deleteShoppingListItems()}>(delete)</InlineButton>
				{:else}
					<a href={`/?product-highlight=${data.id}`}>{link_count} references to shopping list</a>
					<InlineButton onclick={() => deleteShoppingListItems()}>(delete all)</InlineButton>
				{/if}
			</div>
		{:else}
			<InlineButton onclick={() => addShoppingListItem()}>Add to shopping list</InlineButton>
		{/if}
	</div>

	<div class="buttons">
		<ButtonGroup>
			<Button onclick={() => saveChanges()} disabled={!hasChanged}>
				<CheckIcon /> save
			</Button>
			<Button onclick={() => revertChanges()} disabled={!hasChanged}>
				<UndoIcon /> revert
			</Button>
			<Button onclick={() => (confirmDeleteModal = true)}>
				<DeleteIcon /> delete
			</Button>
		</ButtonGroup>
	</div>

	<div class="metadata">
		<div>Created: {data.created.toUTCString()}</div>
		<div>Updated: {data.updated?.toUTCString() ?? 'never'}</div>
	</div>
</section>

{#if confirmDeleteModal}
	<ModalBackdrop onclose={() => (confirmDeleteModal = false)}>
		<Modal size="small">
			<div class="confirmation-modal">
				<div>
					Are you sure you want to delete <i>{data.data.name}</i>?
				</div>

				{#if data.data.shoppingListItemLinks.length > 0}
					<small>
						The product is still on your shopping list. Press "unlink and delete" if you want to
						keep it on your shopping list, but delete the product itself.
					</small>
				{/if}
			</div>

			{#snippet footer()}
				<ButtonGroup>
					{#if data.data.shoppingListItemLinks.length > 0}
						<Button
							onclick={async () => {
								await deleteShoppingListItems();
								await deleteProduct();
								confirmDeleteModal = false;
							}}
						>
							Delete all
						</Button>
						<Button
							onclick={async () => {
								await unlinkShoppingListItems();
								await deleteProduct();
								confirmDeleteModal = false;
							}}
						>
							Unlink and delete
						</Button>
					{:else}
						<Button
							onclick={async () => {
								await deleteProduct();
								confirmDeleteModal = false;
							}}
						>
							Delete
						</Button>
					{/if}

					<Button
						onclick={() => {
							confirmDeleteModal = false;
						}}
					>
						Cancel
					</Button>
				</ButtonGroup>
			{/snippet}
		</Modal>
	</ModalBackdrop>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
	.page .data .property {
		display: flex;
		gap: 0.5em;
	}
	.page .data .property .input {
		flex: 1;
	}
	.page .data .property .input .flex {
		display: flex;
	}
	.page .metadata {
		display: block;
		font-size: 0.8em;
		font-style: italic;
		text-align: right;
		margin: 0.3em 0.4em;
		color: var(--element-color-600);
	}
	.page .statistics {
		display: block;
		font-size: 0.8em;
		text-align: right;
		margin: 0.3em 0.4em;
		color: var(--element-color-500);
	}
	.page .buttons {
		display: flex;
		justify-content: end;
		gap: 1em;
	}
	.confirmation-modal {
		display: flex;
		flex-direction: column;
		padding: 0.9em 1.2em;
		gap: 0.4em;
	}
</style>
