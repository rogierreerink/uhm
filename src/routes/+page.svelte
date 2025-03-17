<script lang="ts">
	import { Box, Dropdown } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		Slot
	} from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon, AddIcon, SubstractIcon } from '$lib/components/icons';
	import { Button } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';
	import { unfoldHeight } from '$lib/transitions';
	import { Label } from '$lib/components/labels';

	const items = [
		{ qty: '1', label: 'hi', isle: 'bloep', checked: true },
		{ qty: '2', label: 'hey', isle: 'bloep', checked: true },
		{ qty: '1', label: 'yo', isle: 'blep', checked: false },
		{ qty: '500gr', label: 'yoo', isle: 'blop', checked: true },
		{ qty: '1l', label: 'wow', isle: 'blup', checked: false },
		{ qty: '1', label: 'wauw', isle: 'blup', checked: false },
		{ qty: '1', label: 'hey', checked: false },
		{ qty: '1', label: 'yo', checked: false },
		{ qty: '1', label: 'yoo', checked: false },
		{ qty: '1', label: 'wow', checked: false },
		{ qty: '1', label: 'wauw', checked: false }
	];

	let qtyDropdownItem = $state<number>();
	let moreDropdownItem = $state<number>();
	let swipedItem = $state<{
		idx: number;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();
</script>

<section class="page">
	<h1>Shopping list</h1>

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
								onclick={() =>
									(qtyDropdownItem = qtyDropdownItem !== itemIdx ? itemIdx : undefined)}
							>
								<TextSlot>{item.qty}</TextSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if qtyDropdownItem === itemIdx}
									<div class="dropdown" style={`z-index: ${items.length + 10 - itemIdx}`}>
										<Box>
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
										</Box>
									</div>
								{/if}
							{/snippet}
						</DropdownSlot>

						<TextSlot fill>
							{item.label}
						</TextSlot>

						{#if item.isle}
							<TextSlot>
								<Label>{item.isle}</Label>
							</TextSlot>
						{/if}

						<DropdownSlot position="to-left">
							<ButtonSlot
								onclick={() =>
									(moreDropdownItem = moreDropdownItem !== itemIdx ? itemIdx : undefined)}
							>
								<IconSlot>
									<MoreIcon />
								</IconSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if moreDropdownItem === itemIdx}
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
		<div class="input-box">
			<Box>
				<div class="input">
					<TextInput placeholder="add item..." />
				</div>
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
	.page .add-item .input-box {
		flex: 1;
	}
	.page .add-item .input-box .input {
		display: flex;
	}
</style>
