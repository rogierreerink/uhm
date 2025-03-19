<script lang="ts">
	import { Box, Dropdown } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot
	} from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon, DeleteIcon } from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';
	import { unfoldHeight } from '$lib/transitions';
	import { page } from '$app/state';
	import { product, products, type ProductsResponse } from '$lib/data/products';
	import { onMount } from 'svelte';
	import { replaceState } from '$app/navigation';

	let data = $state<ProductsResponse>();
	let searchItemInput = $state(page.url.searchParams.get('name') || '');
	let addItemInput = $state('');
	let moreDropdownItem = $state<string>();
	let swipedItem = $state<{
		resourceId: string;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();

	async function getProducts(init?: true) {
		const searchParams: { [key: string]: string } = {};

		const searchName = encodeURI(searchItemInput.trim());
		if (searchName.length > 0) {
			searchParams['name'] = searchName;
		}

		if (!init) {
			replaceState(`?${new URLSearchParams(searchParams).toString()}`, {});
		}

		data = await products.get(searchParams);
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

		getProducts();
	}

	async function deleteProduct(id: string) {
		await product.delete(id);

		getProducts();
	}

	onMount(() => {
		getProducts(true);
	});
</script>

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
			{#if data && (data.data.length || 0) > 0}
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
									deleteProduct(item.id);
								}
								swipedItem = undefined;
							}}
							onclose={() => {
								swipedItem = undefined;
							}}
						>
							<TextSlot fill>
								{item.data.name}
							</TextSlot>

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
										<div
											class="dropdown"
											style={`z-index: ${(data?.data.length || 0) + 10 - itemIdx}`}
										>
											<Dropdown>
												<div transition:unfoldHeight>
													<List>
														<ListItem>
															<ButtonSlot
																onclick={() => {
																	deleteProduct(item.id);
																	moreDropdownItem = undefined;
																}}
															>
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

							{#snippet left()}
								<ButtonSlot
									onclick={() => {
										deleteProduct(item.id);
										swipedItem = undefined;
									}}
								>
									<TextSlot>delete</TextSlot>
								</ButtonSlot>
							{/snippet}
						</SwipeSlot>
					</ListItem>
				{/each}
			{:else}
				<ListItem>
					<TextSlot />
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
