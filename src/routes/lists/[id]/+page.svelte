<script lang="ts">
	import { Box } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		SquareSlot,
		TextButtonSlot,
		TextAnchorSlot
	} from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon } from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { CheckInput, TextInput } from '$lib/components/form';
	import list, { type GetResponse } from '$lib/data/lists/resource';
	import list_items from '$lib/data/lists/items/collection';
	import list_item, { type GetResponse as GetItemResponse } from '$lib/data/lists/items/resource';
	import products, { type GetResponse as GetProductResponse } from '$lib/data/products/collection';
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

	let addItemProductMatches: GetProductResponse | undefined = $state();

	$effect(() => {
		if (addItemInput.length == 0) {
			addItemProductMatches = undefined;
			return;
		}

		let product_matches = products
			.get(
				new URLSearchParams({
					name: addItemInput
				})
			)
			.then((response) => {
				if (response.ok) {
					addItemProductMatches = response.data;
				}
			});
	});

	async function createTemporaryItem() {
		const text = addItemInput.trim();
		if (text.length === 0) {
			return;
		}

		addItemInput = '';

		await list_items.post(data.id, {
			data: [
				{
					kind: {
						type: 'temporary',
						data: { name: text }
					}
				}
			]
		});

		await invalidate(list.url(data.id));
	}

	async function createProductItem(product_id: string) {
		await list_items.post(data.id, {
			data: [
				{
					kind: {
						type: 'product',
						id: product_id
					}
				}
			]
		});

		await invalidate(list.url(data.id));
	}

	async function setChecked(id: string, checked: boolean) {
		await list_item.patch(data.id, id, { checked });
		await invalidate(list.url(data.id));
	}

	async function deleteItem(id: string) {
		await list_item.delete(data.id, id);
		await invalidate(list.url(data.id));
	}

	async function convertToProduct(id: string) {
		const item_ = await list_item.get(data.id, id);
		if (!item_.ok || item_.data.data.kind.type === 'product') {
			return;
		}

		const product_post = await products.post({
			data: [{ name: item_.data.data.kind.data.name }]
		});
		if (!product_post.ok) {
			return;
		}

		await list_item.delete(data.id, item_.data.id);
		await list_items.post(data.id, {
			data: [
				{
					kind: {
						type: 'product',
						id: product_post.data.data[0].id
					}
				}
			]
		});

		await invalidate(list.url(data.id));
	}
</script>

<svelte:head>
	<title>{data.data.name}</title>
</svelte:head>

<section class="page">
	<h1>{data.data.name}</h1>

	<Box>
		<List>
			{#each data.data.items as item, itemIdx (item.id)}
				<ListItem>
					<SwipeSlot
						show={swipedItem && swipedItem?.id === item.id ? swipedItem.area : undefined}
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
									confirmDeleteModal = list_item.get(data.id, item.id);
									break;
								case 'right':
									setChecked(item.id, !item.data.checked);
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
								class:highlight={item.data.kind.type === 'product' &&
									item.data.kind.id === page.url.searchParams.get('product-highlight')}
							>
								{item.data.kind.data.name}
							</div>
						</TextSlot>

						{#if item.data.kind.type === 'temporary'}
							<ButtonSlot onclick={() => convertToProduct(item.id)}>
								<Label>save as product</Label>
							</ButtonSlot>
						{/if}

						<DropdownSlot
							position="to-left"
							show={moreDropdownItem === item.id}
							zIndex={data?.data.items.length + 10 - itemIdx}
							ontoggle={() => {
								moreDropdownItem = moreDropdownItem !== item.id ? item.id : undefined;
							}}
						>
							<IconSlot>
								<MoreIcon />
							</IconSlot>

							{#snippet dropdown()}
								<List>
									{#if item.data.kind.type === 'product'}
										<ListItem>
											<TextAnchorSlot href={`/products/${item.data.kind.id}`} fill>
												view product
											</TextAnchorSlot>
										</ListItem>
									{/if}

									<ListItem>
										<TextButtonSlot
											onclick={() => {
												confirmDeleteModal = list_item.get(data.id, item.id);
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

						<SquareSlot>
							<CheckInput
								checked={item.data.checked}
								oninput={() => setChecked(item.id, !item.data.checked)}
							/>
						</SquareSlot>

						{#snippet left()}
							<TextButtonSlot
								onclick={() => {
									confirmDeleteModal = list_item.get(data.id, item.id);
									swipedItem = undefined;
								}}
							>
								delete
							</TextButtonSlot>
						{/snippet}

						{#snippet right()}
							<TextButtonSlot onclick={() => setChecked(item.id, !item.data.checked)}>
								{item.data.checked ? 'uncheck' : 'check'}
							</TextButtonSlot>
						{/snippet}
					</SwipeSlot>
				</ListItem>
			{/each}

			{#if data.data.items.length === 0}
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

	<div class="add-item-wrapper">
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

		{#if addItemProductMatches && addItemProductMatches.data.length > 0}
			<Box>
				<List>
					{#each addItemProductMatches.data as product (product.id)}
						<ListItem>
							<TextButtonSlot
								onclick={async () => {
									await createProductItem(product.id);
									addItemInput = '';
								}}
								fill>{product.data.name}</TextButtonSlot
							>
						</ListItem>
					{/each}
				</List>
			</Box>
		{/if}
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
							Are you sure you want to delete <i>{data.data.kind.data.name}</i>?
						</div>
					</div>

					{#snippet footer()}
						<ButtonGroup>
							<Button
								onclick={async () => {
									await deleteItem(data.id);
									confirmDeleteModal = undefined;
								}}
							>
								Delete
							</Button>
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
	.page .empty {
		text-align: center;
	}
	.page .add-item-wrapper {
		display: flex;
		flex-direction: column;
		gap: 0.5em;
	}
	.page .add-item-wrapper .add-item {
		display: flex;
		gap: 0.5em;
	}
	.page .add-item-wrapper .add-item .input-box {
		flex: 1;
	}
	.page .add-item-wrapper .add-item .input-box .input {
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
