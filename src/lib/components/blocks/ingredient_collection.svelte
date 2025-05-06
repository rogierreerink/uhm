<script lang="ts" module>
	export type IngredientCollectionData = {
		id: string;
		data: {
			ingredients: {
				id: string;
				data: {
					product: {
						id: string;
						data: {
							name: string;
						};
					};
					list_references: {
						id: string;
						data: {
							name: string;
							items: {
								id: string;
							}[];
						};
					}[];
				};
			}[];
		};
	};

	export type Lists = {
		id: string;
		data: {
			name: string;
		};
	}[];
</script>

<script lang="ts">
	import { Box } from '../boxes';
	import { AddIcon, BasketIcon, BasketAddIcon, DeleteIcon, MoreIcon, CheckIcon } from '../icons';
	import { Label } from '../labels';
	import { List, ListItem } from '../list';
	import {
		DropdownSlot,
		IconButtonSlot,
		IconSlot,
		TextAnchorSlot,
		TextButtonSlot
	} from '../list/slots';
	import { InputBox } from '../form/sections';

	import list_items from '$lib/data/lists/items/collection';
	import list_item from '$lib/data/lists/items/resource';
	import ingredients from '$lib/data/ingredient-collections/ingredients/collection';
	import ingredient from '$lib/data/ingredient-collections/ingredients/resource';
	import products from '$lib/data/products/collection';

	let {
		collection,
		lists,
		oninvalidate
	}: {
		collection: IngredientCollectionData;
		lists: Lists;
		oninvalidate: () => void;
	} = $props();

	type DropdownType = 'basket' | 'more';

	let dropdownItem = $state<{
		type: DropdownType;
		id: string;
	}>();

	function toggleDropdown(type: DropdownType, id: string) {
		if (!dropdownIsShown(type, id)) {
			dropdownItem = {
				type,
				id
			};
		} else {
			dropdownItem = undefined;
		}
	}

	function dropdownIsShown(type: DropdownType, id: string): boolean {
		return dropdownItem?.type === type && dropdownItem?.id === id;
	}

	async function addToList(listId: string, ingredientId: string) {
		await list_items.post(listId, {
			data: [
				{
					kind: {
						type: 'ingredient',
						id: ingredientId
					}
				}
			]
		});
		oninvalidate();
	}

	async function removeFromList(listId: string, itemId: string) {
		await list_item.delete(listId, itemId);
		oninvalidate();
	}

	async function addToCollection(collectionId: string, productId: string): Promise<boolean> {
		let response = await ingredients.post(collectionId, {
			data: [
				{
					product: {
						id: productId
					}
				}
			]
		});
		if (!response.ok) {
			return false;
		}

		oninvalidate();
		return true;
	}

	async function removeFromCollection(collectionId: string, ingredientId: string) {
		await ingredient.delete(collectionId, ingredientId);
		oninvalidate();
	}

	let inputSuggestions = $state<
		{
			id: string;
			text: string;
		}[]
	>([]);

	async function getSuggestions(text: string): Promise<boolean> {
		if (text.length === 0) {
			inputSuggestions = [];
			return false;
		}

		let response = await products.get(
			new URLSearchParams({
				name: text,
				take: '5'
			})
		);

		if (response.ok) {
			inputSuggestions = response.data.data.map((product) => ({
				id: product.id,
				text: product.data.name
			}));
		}

		return true;
	}

	async function addToCollectionFromInput(text: string): Promise<boolean> {
		let suggested = inputSuggestions.find(
			(suggestion) => suggestion.text.toLowerCase() === text.toLowerCase()
		);
		if (!suggested) {
			return false;
		}

		if (!(await addToCollection(collection.id, suggested.id))) {
			return false;
		}

		inputSuggestions = [];
		return true;
	}

	async function addToCollectionFromSuggestion(idx: number): Promise<boolean> {
		let suggested = inputSuggestions[idx];
		if (!suggested) {
			return false;
		}

		if (!(await addToCollection(collection.id, suggested.id))) {
			return false;
		}

		inputSuggestions = [];
		return true;
	}
</script>

<div class="collection">
	<Box>
		<List>
			{#each collection.data.ingredients as ingredient, idx (ingredient.id)}
				<ListItem>
					<TextAnchorSlot fill href={`/products/${ingredient.data.product.id}`}>
						{ingredient.data.product.data.name}
					</TextAnchorSlot>

					<DropdownSlot
						show={dropdownIsShown('basket', ingredient.id)}
						ontoggle={() => toggleDropdown('basket', ingredient.id)}
						position={'to-left'}
						zIndex={10 + collection.data.ingredients.length - idx}
						backdropZIndex={9}
					>
						<IconSlot>
							{#if ingredient.data.list_references.length > 0}
								<Label>
									<BasketIcon />
									{ingredient.data.list_references.length}
								</Label>
							{:else}
								<BasketAddIcon />
							{/if}
						</IconSlot>

						{#snippet dropdown()}
							<List>
								{#each lists as list (list.id)}
									{@const list_ref = ingredient.data.list_references.find(
										(list_ref) => list_ref.id === list.id
									)}

									<ListItem>
										<TextAnchorSlot
											href={`/lists/${list.id}/?ingredient-highlight=${ingredient.id}`}
											fill
										>
											<i>{list.data.name}</i>
										</TextAnchorSlot>

										{#if list_ref}
											<IconButtonSlot
												onclick={() =>
													removeFromList(
														list.id,
														list_ref.data.items[list_ref.data.items.length - 1].id
													)}
												disabled={list_ref.data.items.length == 0}
											>
												<DeleteIcon />
											</IconButtonSlot>
										{:else}
											<IconButtonSlot onclick={() => addToList(list.id, ingredient.id)}>
												<AddIcon />
											</IconButtonSlot>
										{/if}
									</ListItem>
								{/each}
							</List>
						{/snippet}
					</DropdownSlot>

					<DropdownSlot
						show={dropdownIsShown('more', ingredient.id)}
						ontoggle={() => toggleDropdown('more', ingredient.id)}
						position={'to-left'}
						zIndex={10 + collection.data.ingredients.length - idx}
						backdropZIndex={9}
					>
						<IconSlot>
							<MoreIcon />
						</IconSlot>

						{#snippet dropdown()}
							<List>
								<ListItem>
									<TextButtonSlot
										onclick={() => removeFromCollection(collection.id, ingredient.id)}
									>
										Delete
									</TextButtonSlot>
								</ListItem>
							</List>
						{/snippet}
					</DropdownSlot>
				</ListItem>
			{/each}
		</List>
	</Box>

	<InputBox
		placeholder="add ingredient..."
		suggestions={inputSuggestions}
		oninput={(text) => getSuggestions(text)}
		onenter={(text) => addToCollectionFromInput(text)}
		onentersuggestion={(idx) => addToCollectionFromSuggestion(idx)}
	/>
</div>

<style>
	.collection {
		display: flex;
		flex-direction: column;
		gap: 0.5em;
	}
</style>
