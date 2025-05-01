<script lang="ts">
	import { goto, invalidate } from '$app/navigation';
	import Box from '$lib/components/boxes/box.svelte';
	import { Breadcrumb, BreadcrumbTrail } from '$lib/components/breadcrumb';
	import { Label, TextInput } from '$lib/components/form';
	import { Button, ButtonGroup, InlineButton } from '$lib/components/form/buttons';
	import { CheckIcon, DeleteIcon, UndoIcon } from '$lib/components/icons';
	import { Modal, ModalBackdrop } from '$lib/components/modal';
	import product, { type GetResponse } from '$lib/data/products/resource';
	import list_items from '$lib/data/lists/items/collection';
	import list_item from '$lib/data/lists/items/resource';
	import { type GetResponse as GetListsResponse } from '$lib/data/lists/collection';

	let {
		data
	}: {
		data: {
			product: GetResponse;
			lists: GetListsResponse;
		};
	} = $props();

	let name = $state(data.product.data.name);
	let hasChanged = $derived(name !== data.product.data.name);
	let confirmDeleteModal = $state(false);

	async function saveChanges() {
		await product.patch(data.product.id, { name });
		await invalidate(product.url(data.product.id));
	}

	async function revertChanges() {
		name = data.product.data.name;
	}

	async function deleteProduct() {
		await product.delete(data.product.id);
		goto('.', { replaceState: true });
	}

	async function unlinkListReferences() {
		const response = await product.get(data.product.id);
		if (!response.ok) {
			return;
		}

		for (const { id, data } of response.data.data.list_item_references) {
			const list_id = data.list_reference.id;
			await list_items.post(list_id, {
				data: [
					{
						kind: {
							type: 'temporary',
							data: {
								name: response.data.data.name
							}
						}
					}
				]
			});
			await list_item.delete(list_id, id);
		}

		await invalidate(product.url(data.product.id));
	}

	async function addToList(list_id: string, item_id: string) {
		await list_items.post(list_id, {
			data: [
				{
					kind: {
						type: 'product',
						id: item_id
					}
				}
			]
		});

		await invalidate(product.url(data.product.id));
	}

	async function removeFromList(list_id: string, item_id: string) {
		await list_item.delete(list_id, item_id);

		await invalidate(product.url(data.product.id));
	}
</script>

<svelte:head>
	<title>Products: {data.product.data.name.toLocaleLowerCase()}</title>
</svelte:head>

<section class="page">
	<h1>{data.product.data.name}</h1>

	<BreadcrumbTrail>
		<Breadcrumb href=".">products</Breadcrumb>
		<Breadcrumb>{data.product.data.name.toLocaleLowerCase()}</Breadcrumb>
	</BreadcrumbTrail>

	<div class="data">
		<div class="property">
			<Label for="name">Name</Label>
			<div class="input">
				<Box>
					<div class="flex">
						<TextInput
							id="name"
							placeholder={data.product.data.name}
							value={name}
							oninput={(e) => (name = e.currentTarget.value.trim())}
						/>
					</div>
				</Box>
			</div>
		</div>
	</div>

	<div class="statistics">
		{#each data.lists.data as list (list.id)}
			{@const list_refs = data.product.data.list_item_references.filter(
				(ref) => ref.data.list_reference.data.name == list.data.name
			)}
			<!-- {@const plural = new Intl.PluralRules('en-US').select(list_refs.length) !== 'one'} -->
			<div class="buttons">
				<a href={`/lists/${list.id}/?product-highlight=${data.product.id}`}>
					{#if list_refs.length > 0}
						{list_refs.length} on
					{/if}
					{list.data.name}
				</a>
				-
				<span class:disabled={list_refs.length == 0}>
					<InlineButton
						onclick={() => removeFromList(list.id, list_refs[list_refs.length - 1].id)}
						disabled={list_refs.length == 0}
					>
						remove
					</InlineButton>
				</span>
				/
				<InlineButton onclick={() => addToList(list.id, data.product.id)}>add</InlineButton>
			</div>
		{/each}
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
		<div>Created: {data.product.ts_created.toUTCString()}</div>
		<div>Updated: {data.product.ts_updated?.toUTCString() ?? 'never'}</div>
	</div>
</section>

{#if confirmDeleteModal}
	<ModalBackdrop onclose={() => (confirmDeleteModal = false)}>
		<Modal size="small">
			<div class="confirmation-modal">
				<div>
					Are you sure you want to delete <i>{data.product.data.name}</i>?
				</div>

				{#if data.product.data.list_item_references.length > 0}
					<small>
						The product is still on some of your lists. Press "unlink and delete" if you want to
						keep it there while deleting the product itself.
					</small>
				{/if}
			</div>

			{#snippet footer()}
				<ButtonGroup>
					{#if data.product.data.list_item_references.length > 0}
						<Button
							onclick={async () => {
								await deleteProduct();
								confirmDeleteModal = false;
							}}
						>
							Delete all
						</Button>
						<Button
							onclick={async () => {
								await unlinkListReferences();
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
	.page .statistics .buttons {
		all: unset;
		display: block;
		user-select: none;
	}
	.page .statistics .buttons .disabled {
		color: var(--element-color-600);
		font-style: italic;
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
