<script lang="ts">
	import { Box, Dropdown } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		Slot,
		SquareSlot
	} from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon, AddIcon, SubstractIcon } from '$lib/components/icons';
	import { Button } from '$lib/components/form/buttons';
	import { CheckInput, TextInput } from '$lib/components/form';
	import { unfoldHeight } from '$lib/transitions';
	import { Label } from '$lib/components/labels';
	import {
		shoppingListItem,
		shoppingListItems,
		type ShoppingListItemsResponse
	} from '$lib/data/shopping-list';
	import { invalidate } from '$app/navigation';

	let {
		data
	}: {
		data: ShoppingListItemsResponse;
	} = $props();

	let addItemInput = $state('');
	let qtyDropdownItem = $state<string>();
	let moreDropdownItem = $state<string>();
	let swipedItem = $state<{
		id: string;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();

	async function createTemporaryItem() {
		const text = addItemInput.trim();
		if (text.length === 0) {
			return;
		}

		addItemInput = '';

		await shoppingListItems.post({
			data: [
				{
					source: {
						type: 'temporary',
						data: { name: text }
					}
				}
			]
		});
		await invalidate(shoppingListItems.url());
	}

	async function setInCart(id: string, inCart: boolean) {
		await shoppingListItem.patch(id, { inCart });
		await invalidate(shoppingListItem.url(id));
		await invalidate(shoppingListItems.url());
	}

	async function deleteItem(id: string) {
		await shoppingListItem.delete(id);
		await invalidate(shoppingListItem.url(id));
		await invalidate(shoppingListItems.url());
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
									deleteItem(item.id);
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
							{item.data.source.data.name}
						</TextSlot>

						<!-- {#if item.isle}
							<TextSlot>
								<Label>{item.isle}</Label>
							</TextSlot>
						{/if} -->

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
													<ListItem>
														<ButtonSlot onclick={() => deleteItem(item.id)}>
															<TextSlot>delete</TextSlot>
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
							<ButtonSlot onclick={() => deleteItem(item.id)}>
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

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
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
</style>
