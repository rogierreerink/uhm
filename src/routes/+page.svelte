<script lang="ts">
	import { Box, Dropdown } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		SquareSlot,
		AnchorSlot
	} from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon } from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { CheckInput, TextInput } from '$lib/components/form';
	import { unfoldHeight } from '$lib/transitions';
	import shoppingList, { type GetResponse } from '$lib/data/shopping-list/collection';
	import shoppingListItem, {
		type GetResponse as GetItemResponse
	} from '$lib/data/shopping-list/resource';
	import products from '$lib/data/products/collection';
	import { invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import Label from '$lib/components/labels/label.svelte';
	import type { DataResponse } from '$lib/data';
	import { Modal, ModalBackdrop } from '$lib/components/modal';

	let {
		data
	}: {
		data: GetResponse;
	} = $props();

	let addItemInput = $state('');
	let qtyDropdownItem = $state<string>();
	let moreDropdownItem = $state<string>();
	let swipedItem = $state<{
		id: string;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();

	let confirmDeleteModal = $state<Promise<DataResponse<GetItemResponse>>>();

	async function createTemporaryItem() {
		const text = addItemInput.trim();
		if (text.length === 0) {
			return;
		}

		addItemInput = '';

		await shoppingList.post({
			data: [
				{
					source: {
						type: 'temporary',
						data: { name: text }
					}
				}
			]
		});
		await invalidate(shoppingList.url());
	}

	async function setInCart(id: string, inCart: boolean) {
		await shoppingListItem.patch(id, { inCart });
		await invalidate(shoppingList.url());
	}

	async function deleteItem(id: string) {
		await shoppingListItem.delete(id);
		await invalidate(shoppingList.url());
	}

	async function convertToProduct(id: string) {
		const item = await shoppingListItem.get(id);
		if (!item.ok || item.data.data.source.type === 'product') {
			return;
		}

		const product_post = await products.post({
			data: [{ name: item.data.data.source.data.name }]
		});
		if (!product_post.ok) {
			return;
		}

		await shoppingListItem.patch(id, {
			source: {
				type: 'product',
				id: product_post.data.data[0].id
			}
		});
		await invalidate(shoppingList.url());
	}
</script>

<svelte:head>
	<title>Shopping list</title>
</svelte:head>

<section class="page">
	<h1>Shopping list</h1>

	<Box>
		<List>
			{#each data.data as item, itemIdx (item.id)}
				<ListItem>
					<SwipeSlot
						show={swipedItem?.id === item.id ? swipedItem.area : undefined}
						onshow={(area) => {
							swipedItem = {
								id: item.id,
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
							switch (swipedItem?.area) {
								case 'left':
									confirmDeleteModal = shoppingListItem.get(item.id);
									break;
								case 'right':
									setInCart(item.id, !item.data.inCart);
									break;
							}
							swipedItem = undefined;
						}}
						onclose={() => {
							swipedItem = undefined;
						}}
					>
						<!-- <DropdownSlot>
							<ButtonSlot
								onclick={() =>
									(qtyDropdownItem = qtyDropdownItem !== itemIdx ? itemIdx : undefined)}
							>
								<TextSlot>{item.qty}</TextSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if qtyDropdownItem === itemIdx}
									<div class="dropdown" style={`z-index: ${items.length + 10 - itemIdx}`}>
										<Dropdown>
											<div transition:unfoldHeight>
												<List>
													<ListItem>
														<ButtonSlot>
															<IconSlot>
																<SubstractIcon />
															</IconSlot>
														</ButtonSlot>
														<Slot>
															<TextInput size={5} value={item.qty} />
														</Slot>
														<ButtonSlot>
															<IconSlot>
																<AddIcon />
															</IconSlot>
														</ButtonSlot>
													</ListItem>
												</List>
											</div>
										</Dropdown>
									</div>
								{/if}
							{/snippet}
						</DropdownSlot> -->

						<TextSlot fill>
							<div
								class:highlight={item.data.source.type === 'product' &&
									item.data.source.id === page.url.searchParams.get('product-highlight')}
							>
								{item.data.source.data.name}
							</div>
						</TextSlot>

						<!-- {#if item.isle}
							<TextSlot>
								<Label>{item.isle}</Label>
							</TextSlot>
						{/if} -->

						{#if item.data.source.type === 'temporary'}
							<ButtonSlot onclick={() => convertToProduct(item.id)}>
								<Label>save as product</Label>
							</ButtonSlot>
						{/if}

						<DropdownSlot position="to-left">
							<ButtonSlot
								onclick={() =>
									(moreDropdownItem = moreDropdownItem !== item.id ? item.id : undefined)}
							>
								<IconSlot>
									<MoreIcon />
								</IconSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if moreDropdownItem === item.id}
									<div class="dropdown" style={`z-index: ${data.data.length + 10 - itemIdx}`}>
										<Dropdown>
											<div transition:unfoldHeight>
												<List>
													{#if item.data.source.type === 'product'}
														<ListItem>
															<AnchorSlot href={`/products/${item.data.source.id}`} fill>
																<TextSlot fill>view product</TextSlot>
															</AnchorSlot>
														</ListItem>
													{/if}

													<ListItem>
														<ButtonSlot
															onclick={() => {
																confirmDeleteModal = shoppingListItem.get(item.id);
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

						<SquareSlot>
							<CheckInput
								checked={item.data.inCart}
								oninput={() => setInCart(item.id, !item.data.inCart)}
							/>
						</SquareSlot>

						{#snippet left()}
							<ButtonSlot
								onclick={() => {
									confirmDeleteModal = shoppingListItem.get(item.id);
									swipedItem = undefined;
								}}
							>
								<TextSlot>delete</TextSlot>
							</ButtonSlot>
						{/snippet}

						{#snippet right()}
							<ButtonSlot onclick={() => setInCart(item.id, !item.data.inCart)}>
								<TextSlot>
									{item.data.inCart ? 'uncheck' : 'check'}
								</TextSlot>
							</ButtonSlot>
						{/snippet}
					</SwipeSlot>
				</ListItem>
			{/each}

			{#if data.data.length === 0}
				<ListItem>
					<TextSlot fill>
						<div class="empty">
							<i>empty</i>
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
								createTemporaryItem();
							}
						}}
					/>
				</div>
			</Box>
		</div>
		<Button onclick={() => createTemporaryItem()}>
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
							Are you sure you want to delete <i>{data.data.source.data.name}</i>?<br />
						</div>
					</div>

					{#snippet footer()}
						<ButtonGroup>
							<Button
								onclick={async () => {
									await deleteItem(data.id);
									confirmDeleteModal = undefined;
								}}>Delete</Button
							>
							<Button
								onclick={() => {
									confirmDeleteModal = undefined;
								}}>Cancel</Button
							>
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
	.page .empty {
		text-align: center;
	}
	.page .dropdown {
		position: relative;
		margin-right: -1px;
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
	.page .highlight {
		animation-name: item-highlight;
		animation-duration: 3s;
		animation-timing-function: ease;
	}
	@keyframes item-highlight {
		10% {
			color: white;
		}
	}
	.confirmation-modal {
		display: flex;
		flex-direction: column;
		padding: 0.9em 1.2em;
		gap: 0.4em;
	}
</style>
