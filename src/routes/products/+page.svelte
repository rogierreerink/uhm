<script lang="ts">
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { Box, Dropdown } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		AnchorSlot
	} from '$lib/components/list/slots';
	import {
		MoreIcon,
		CheckIcon,
		DeleteIcon,
		BasketAddIcon,
		BasketIcon
	} from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';
	import { unfoldHeight } from '$lib/transitions';
	import { Label } from '$lib/components/labels';
	import shoppingList from '$lib/data/shopping-list/collection';
	import shoppingListItem from '$lib/data/shopping-list/resource';
	import products, { type GetResponse } from '$lib/data/products/collection';
	import product, { type GetResponse as GetProductResponse } from '$lib/data/products/resource';
	import { Modal, ModalBackdrop } from '$lib/components/modal';

	let {
		data
	}: {
		data: GetResponse;
	} = $props();

	let searchItemInput = $state(page.url.searchParams.get('name') || '');
	let addItemInput = $state('');
	let moreDropdownItem = $state<string>();
	let swipedItem = $state<{
		resourceId: string;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();

	let confirmDeleteModal = $state<Promise<GetProductResponse>>();

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
		await invalidate(product.url(id));
	}

	async function unlinkShoppingListItems(id: string) {
		const prod = await product.get(id);
		for (const { id } of prod.data.shopping_list_item_links) {
			await shoppingListItem.patch(id, {
				source: {
					type: 'temporary',
					data: {
						name: prod.data.name
					}
				}
			});
		}
		await invalidate(products.url(page.url.searchParams));
		await invalidate(product.url(id));
		await invalidate(shoppingList.url());
	}

	async function addToShoppingList(id: string) {
		await shoppingList.post({
			data: [
				{
					source: {
						type: 'product',
						id
					}
				}
			]
		});
		await invalidate(products.url(page.url.searchParams));
		await invalidate(product.url(id));
		await invalidate(shoppingList.url());
	}

	async function deleteShoppingListItems(id: string) {
		const prod = await product.get(id);
		for (const { id } of prod.data.shopping_list_item_links) {
			await shoppingListItem.delete(id);
		}
		await invalidate(products.url(page.url.searchParams));
		await invalidate(product.url(id));
		await invalidate(shoppingList.url());
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
			{#each data.data as item, itemIdx (item.id)}
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
						<AnchorSlot fill href={`/products/${item.id}`}>
							<TextSlot fill>
								{item.data.name}
							</TextSlot>
						</AnchorSlot>

						{#if item.data.shopping_list_item_links.length > 0}
							<AnchorSlot href={`/?product-highlight=${item.id}`}>
								<TextSlot>
									<Label><BasketIcon /> {item.data.shopping_list_item_links.length}</Label>
								</TextSlot>
							</AnchorSlot>
						{:else}
							<ButtonSlot onclick={() => addToShoppingList(item.id)}>
								<IconSlot>
									<BasketAddIcon />
								</IconSlot>
							</ButtonSlot>
						{/if}

						<DropdownSlot position="to-left">
							<ButtonSlot
								onclick={() => {
									moreDropdownItem = moreDropdownItem !== item.id ? item.id : undefined;
								}}
							>
								<IconSlot>
									<MoreIcon />
								</IconSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if moreDropdownItem === item.id}
									<div class="dropdown" style={`z-index: ${data?.data.length + 10 - itemIdx}`}>
										<Dropdown>
											<div transition:unfoldHeight>
												<List>
													<ListItem>
														<AnchorSlot href={`/products/${item.id}`} fill>
															<TextSlot fill>view</TextSlot>
														</AnchorSlot>
													</ListItem>
													<ListItem>
														<ButtonSlot
															onclick={() => {
																confirmDeleteModal = product.get(item.id);
																moreDropdownItem = undefined;
															}}
															fill
														>
															<TextSlot fill>delete</TextSlot>
														</ButtonSlot>
													</ListItem>
												</List>
											</div>
										</Dropdown>
									</div>
								{/if}
							{/snippet}
						</DropdownSlot>

						{#snippet left()}
							<ButtonSlot
								onclick={() => {
									confirmDeleteModal = product.get(item.id);
									swipedItem = undefined;
								}}
							>
								<TextSlot>delete</TextSlot>
							</ButtonSlot>
						{/snippet}
					</SwipeSlot>
				</ListItem>
			{/each}

			{#if data.data.length === 0}
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
		{#await confirmDeleteModal then data}
			<Modal size="small">
				<div class="confirmation-modal">
					<div>
						Are you sure you want to delete <i>{data.data.name}</i>?<br />
					</div>

					{#if data.data.shopping_list_item_links.length > 0}
						<small>
							The product is still on your shopping list. Press "unlink and delete" if you want to
							keep it on your shopping list, but delete the product itself.
						</small>
					{/if}
				</div>

				{#snippet footer()}
					<ButtonGroup>
						{#if data.data.shopping_list_item_links.length > 0}
							<Button
								onclick={async () => {
									await deleteShoppingListItems(data.id);
									await deleteProduct(data.id);
									confirmDeleteModal = undefined;
								}}>Delete all</Button
							>
							<Button
								onclick={async () => {
									await unlinkShoppingListItems(data.id);
									await deleteProduct(data.id);
									confirmDeleteModal = undefined;
								}}>Unlink and delete</Button
							>
						{:else}
							<Button
								onclick={async () => {
									await deleteProduct(data.id);
									confirmDeleteModal = undefined;
								}}>Delete</Button
							>
						{/if}

						<Button
							onclick={() => {
								confirmDeleteModal = undefined;
							}}>Cancel</Button
						>
					</ButtonGroup>
				{/snippet}
			</Modal>
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
	.page .dropdown {
		position: relative;
		margin-right: -1px;
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
