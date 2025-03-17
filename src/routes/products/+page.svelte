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
	import { MoreIcon, CheckIcon } from '$lib/components/icons';
	import { Button } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';
	import { unfoldHeight } from '$lib/transitions';

	const items = [
		{ label: 'hi', checked: true },
		{ label: 'hey', checked: true },
		{ label: 'yo', checked: false },
		{ label: 'yoo', checked: true },
		{ label: 'wow', checked: false },
		{ label: 'wauw', checked: false },
		{ label: 'hey', checked: false },
		{ label: 'yo', checked: false },
		{ label: 'yoo', checked: false },
		{ label: 'wow', checked: false },
		{ label: 'wauw', checked: false }
	];

	let dropdownItem = $state<number>();
	let swipedItem = $state<{
		idx: number;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();
</script>

<section class="page">
	<h1>Products</h1>

	<Box>
		<List>
			{#each items as item, itemIdx}
				<ListItem>
					<SwipeSlot
						show={swipedItem?.idx === itemIdx ? swipedItem.area : undefined}
						onshow={(area) => (swipedItem = { idx: itemIdx, area })}
						onpretrigger={() => (swipedItem = swipedItem && { ...swipedItem, pretriggered: true })}
						onpretriggerrevert={() =>
							(swipedItem = swipedItem && { ...swipedItem, pretriggered: false })}
						ontrigger={() => console.log('trigger')}
						onclose={() => (swipedItem = undefined)}
					>
						<DropdownSlot>
							<ButtonSlot
								onclick={() => (dropdownItem = dropdownItem !== itemIdx ? itemIdx : undefined)}
							>
								<IconSlot>
									<MoreIcon />
								</IconSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if dropdownItem === itemIdx}
									<div class="dropdown" style={`z-index: ${items.length + 10 - itemIdx}`}>
										<Dropdown>
											<div transition:unfoldHeight>
												<List>
													<ListItem>
														<ButtonSlot>
															<TextSlot>details</TextSlot>
														</ButtonSlot>
													</ListItem>
													<ListItem>
														<ButtonSlot>
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

						<TextSlot fill>
							{item.label}
						</TextSlot>

						<ButtonSlot>
							<IconSlot>
								<CheckIcon enable={item.checked} />
							</IconSlot>
						</ButtonSlot>

						{#snippet left()}
							<ButtonSlot>
								<TextSlot>delete</TextSlot>
							</ButtonSlot>
						{/snippet}

						{#snippet right()}
							<ButtonSlot>
								<TextSlot>
									{item.checked ? 'uncheck' : 'check'}
								</TextSlot>
							</ButtonSlot>
						{/snippet}
					</SwipeSlot>
				</ListItem>
			{/each}
		</List>
	</Box>

	<div class="add-item">
		<div class="input">
			<Box>
				<TextInput placeholder="add item..." />
			</Box>
		</div>
		<Button>
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
		margin-left: -1px;
	}
	.page .add-item {
		display: flex;
		gap: 0.5em;
	}
	.page .add-item .input {
		flex: 1;
	}
</style>
