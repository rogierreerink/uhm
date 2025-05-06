<script lang="ts">
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { Box } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		TextAnchorSlot,
		IconButtonSlot,
		TextButtonSlot
	} from '$lib/components/list/slots';
	import {
		MoreIcon,
		CheckIcon,
		DeleteIcon,
		BasketAddIcon,
		BasketIcon,
		AddIcon
	} from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';
	import { Label } from '$lib/components/labels';
	import list_items from '$lib/data/lists/items/collection';
	import list_item from '$lib/data/lists/items/resource';
	import products, { type GetResponse } from '$lib/data/products/collection';
	import product, { type GetResponse as GetProductResponse } from '$lib/data/products/resource';
	import { type GetResponse as GetListsResponse } from '$lib/data/lists/collection';
	import { Modal, ModalBackdrop } from '$lib/components/modal';
	import type { DataResponse } from '$lib/data';
	import SubstractIcon from '$lib/components/icons/substract-icon.svelte';

	let {
		data
	}: {
		data: {
			products: GetResponse;
			lists: GetListsResponse;
		};
	} = $props();

	let searchItemInput = $state(page.url.searchParams.get('name') || '');
	let addItemInput = $state('');
	let moreDropdownItem = $state<string>();
	let basketDropdownItem = $state<string>();
	let swipedItem = $state<{
		resourceId: string;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();

	let confirmDeleteModal = $state<Promise<DataResponse<GetProductResponse>>>();

	async function getProducts() {
		const searchParams = new URLSearchParams();

		const searchName = encodeURI(searchItemInput.trim());
		if (searchName.length > 0) {
			searchParams.set('name', searchName);
		}

		goto(`?${searchParams.toString()}`, {
			replaceState: true,
			keepFocus: true
		});
	}

	async function createProduct() {
		const text = addItemInput.trim();
		if (text.length === 0) {
			return;
		}

		addItemInput = '';

		await products.post({
			data: [{ name: text }]
		});
		await invalidate(products.url(page.url.searchParams));
	}

	async function deleteProduct(id: string) {
		await product.delete(id);
		await invalidate(products.url(page.url.searchParams));
	}

	async function unlinkListReferences(id: string) {
		const response = await product.get(id);
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

		await invalidate(products.url(page.url.searchParams));
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

		await invalidate(products.url(page.url.searchParams));
	}

	async function removeFromList(list_id: string, item_id: string) {
		await list_item.delete(list_id, item_id);

		await invalidate(products.url(page.url.searchParams));
	}
</script>

<svelte:head>
	<title>Products</title>
</svelte:head>

<section class="page">
	<h1>Products</h1>

	<div class="search">
		<div class="input">
			<Box>
				<div class="stretch">
					<TextInput
						placeholder="search items..."
						value={searchItemInput}
						oninput={(e) => {
							searchItemInput = e.currentTarget.value;
							getProducts();
						}}
					/>
				</div>
			</Box>
		</div>
		<div class="button">
			<ButtonGroup>
				<Button
					onclick={() => {
						searchItemInput = '';
						getProducts();
					}}
				>
					<DeleteIcon />
				</Button>
			</ButtonGroup>
		</div>
	</div>

	<Box>
		<List>
			{#each data.products.data as item, itemIdx (item.id)}
				<ListItem>
					<SwipeSlot
						show={swipedItem?.resourceId === item.id ? swipedItem.area : undefined}
						onshow={(area) => {
							if (area === 'left')
								swipedItem = {
									resourceId: item.id,
									area
								};
						}}
						onpretrigger={() => {
							if (swipedItem) {
								swipedItem = { ...swipedItem, pretriggered: true };
							}
						}}
						onpretriggerrevert={() => {
							if (swipedItem) {
								swipedItem = { ...swipedItem, pretriggered: false };
							}
						}}
						ontrigger={() => {
							if (swipedItem?.area === 'left') {
								confirmDeleteModal = product.get(item.id);
							}
							swipedItem = undefined;
						}}
						onclose={() => {
							swipedItem = undefined;
						}}
					>
						<TextAnchorSlot fill href={`/products/${item.id}`}>
							{item.data.name}
						</TextAnchorSlot>

						<DropdownSlot
							position="to-left"
							show={basketDropdownItem === item.id}
							zIndex={data?.products.data.length + 10 - itemIdx}
							ontoggle={() => {
								basketDropdownItem = basketDropdownItem !== item.id ? item.id : undefined;
								moreDropdownItem = undefined;
							}}
						>
							<IconSlot>
								{#if item.data.list_item_references.length > 0}
									<Label><BasketIcon /> {item.data.list_item_references.length}</Label>
								{:else}
									<BasketAddIcon />
								{/if}
							</IconSlot>

							{#snippet dropdown()}
								<List>
									{#each data.lists.data as list (list.id)}
										{@const list_refs = item.data.list_item_references.filter(
											(ref) => ref.data.list_reference.data.name == list.data.name
										)}

										<ListItem>
											<TextAnchorSlot href={`/lists/${list.id}/?product-highlight=${item.id}`} fill>
												<i>
													{#if list_refs.length > 0}
														{list_refs.length} on
													{/if}
													{list.data.name}
												</i>
											</TextAnchorSlot>

											<IconButtonSlot
												onclick={() => removeFromList(list.id, list_refs[list_refs.length - 1].id)}
												disabled={list_refs.length == 0}
											>
												<SubstractIcon />
											</IconButtonSlot>

											<IconButtonSlot onclick={() => addToList(list.id, item.id)}>
												<AddIcon />
											</IconButtonSlot>
										</ListItem>
									{/each}
								</List>
							{/snippet}
						</DropdownSlot>

						<DropdownSlot
							position="to-left"
							show={moreDropdownItem === item.id}
							zIndex={data?.products.data.length + 10 - itemIdx}
							ontoggle={() => {
								moreDropdownItem = moreDropdownItem !== item.id ? item.id : undefined;
								basketDropdownItem = undefined;
							}}
						>
							<IconSlot>
								<MoreIcon />
							</IconSlot>

							{#snippet dropdown()}
								<List>
									<ListItem>
										<TextAnchorSlot href={`/products/${item.id}`} fill>view</TextAnchorSlot>
									</ListItem>
									<ListItem>
										<TextButtonSlot
											onclick={() => {
												confirmDeleteModal = product.get(item.id);
												moreDropdownItem = undefined;
											}}
											fill
										>
											delete
										</TextButtonSlot>
									</ListItem>
								</List>
							{/snippet}
						</DropdownSlot>

						{#snippet left()}
							<TextButtonSlot
								onclick={() => {
									confirmDeleteModal = product.get(item.id);
									swipedItem = undefined;
								}}
							>
								delete
							</TextButtonSlot>
						{/snippet}
					</SwipeSlot>
				</ListItem>
			{/each}

			{#if data.products.data.length === 0}
				<ListItem>
					<TextSlot fill>
						<div class="not-found">
							<i>no products found</i>
						</div>
					</TextSlot>
				</ListItem>
			{/if}
		</List>
	</Box>

	<div class="add-item">
		<div class="input-box">
			<Box>
				<div class="input">
					<TextInput
						placeholder="add item..."
						value={addItemInput}
						oninput={(e) => {
							addItemInput = e.currentTarget.value;
						}}
						onkeypress={(e) => {
							if (e.key === 'Enter') {
								createProduct();
							}
						}}
					/>
				</div>
			</Box>
		</div>
		<Button onclick={() => createProduct()}>
			<CheckIcon />
		</Button>
	</div>
</section>

{#if confirmDeleteModal}
	<ModalBackdrop onclose={() => (confirmDeleteModal = undefined)}>
		{#await confirmDeleteModal then response}
			{#if response.ok}
				{@const data = response.data}

				<Modal size="small">
					<div class="confirmation-modal">
						<div>
							Are you sure you want to delete <i>{data.data.name}</i>?
						</div>

						{#if data.data.list_item_references.length > 0}
							<small>
								The product is still on some of your lists. Press "unlink and delete" if you want to
								keep it there while deleting the product itself.
							</small>
						{/if}
					</div>

					{#snippet footer()}
						<ButtonGroup>
							{#if data.data.list_item_references.length > 0}
								<Button
									onclick={async () => {
										await deleteProduct(data.id);
										confirmDeleteModal = undefined;
									}}
								>
									Delete all
								</Button>
								<Button
									onclick={async () => {
										await unlinkListReferences(data.id);
										await deleteProduct(data.id);
										confirmDeleteModal = undefined;
									}}
								>
									Unlink and delete
								</Button>
							{:else}
								<Button
									onclick={async () => {
										await deleteProduct(data.id);
										confirmDeleteModal = undefined;
									}}
								>
									Delete
								</Button>
							{/if}

							<Button
								onclick={() => {
									confirmDeleteModal = undefined;
								}}
							>
								Cancel
							</Button>
						</ButtonGroup>
					{/snippet}
				</Modal>
			{/if}
		{/await}
	</ModalBackdrop>
{/if}

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
	.page .search {
		display: flex;
		gap: 0.5em;
	}
	.page .search .input {
		flex: 1;
	}
	.page .search .input .stretch {
		display: flex;
	}
	.page .search .button {
		display: flex;
	}
	.page .not-found {
		text-align: center;
	}
	.page .add-item {
		display: flex;
		gap: 0.5em;
	}
	.page .add-item .input-box {
		flex: 1;
	}
	.page .add-item .input-box .input {
		display: flex;
	}
	.confirmation-modal {
		display: flex;
		flex-direction: column;
		padding: 0.9em 1.2em;
		gap: 0.4em;
	}
</style>
