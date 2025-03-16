<script lang="ts">
	import { Border } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		Slot,
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot
	} from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon, DeleteIcon } from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';
	import { Modal, ModalBackdrop } from '$lib/components/modal';
	import { unfoldHeight, unfoldWidth } from '$lib/transitions';

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

	const options = [{ label: 'options 1' }, { label: 'options 2' }, { label: 'options 3' }];

	let showModal = $state(false);
	let showItemDropdown = $state<number>();
	let showItemSwiped = $state<{
		idx: number;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();

	$inspect(showItemSwiped);
</script>

<section class="page">
	<div class="search">
		<div class="input">
			<Border>
				<TextInput placeholder="search items..." />
			</Border>
		</div>
		<div class="button">
			<ButtonGroup>
				<Button>
					<DeleteIcon />
				</Button>
				<Button>
					<CheckIcon />
				</Button>
			</ButtonGroup>
		</div>
	</div>

	<Border>
		<List>
			{#each items as item, itemIdx}
				<ListItem>
					<SwipeSlot
						show={showItemSwiped?.idx === itemIdx ? showItemSwiped.area : undefined}
						onshow={(area) => (showItemSwiped = { idx: itemIdx, area })}
						onpretrigger={() => {
							if (showItemSwiped) showItemSwiped = { ...showItemSwiped, pretriggered: true };
						}}
						onpretriggerrevert={() => {
							if (showItemSwiped) showItemSwiped = { ...showItemSwiped, pretriggered: false };
						}}
						ontrigger={() => console.log('trigger')}
						onclose={() => (showItemSwiped = undefined)}
					>
						<DropdownSlot>
							<ButtonSlot
								onclick={() =>
									(showItemDropdown = showItemDropdown !== itemIdx ? itemIdx : undefined)}
							>
								<IconSlot>
									<MoreIcon />
								</IconSlot>
							</ButtonSlot>

							{#snippet dropdown()}
								{#if showItemDropdown === itemIdx}
									<div class="dropdown" style={`z-index: ${items.length + 10 - itemIdx}`}>
										<Border --box-bg-color="var(--color-zinc-800)">
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
										</Border>
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
								<TextSlot>
									{showItemSwiped?.pretriggered ? 'delete !' : 'delete'}
								</TextSlot>
							</ButtonSlot>
						{/snippet}

						{#snippet right()}
							<ButtonSlot>
								<TextSlot>
									{showItemSwiped?.pretriggered ? '!' : ''}
									{item.checked ? 'uncheck' : 'check'}
								</TextSlot>
							</ButtonSlot>
						{/snippet}
					</SwipeSlot>
				</ListItem>
			{/each}

			<ListItem>
				<Slot fill>
					<TextInput placeholder="add item" />
				</Slot>
			</ListItem>
		</List>
	</Border>

	<Border>
		<List>
			{#each options as option}
				<ListItem>
					<ButtonSlot fill>
						<TextSlot fill>
							{option.label}
						</TextSlot>
					</ButtonSlot>
				</ListItem>
			{/each}
		</List>
	</Border>

	<div class="buttons">
		<ButtonGroup orientation="horizontal">
			<Button onclick={() => (showModal = !showModal)}>toggle modal</Button>
			<Button>
				<CheckIcon /> ok
			</Button>
			<Button>hola</Button>
			<Button>
				<DeleteIcon /> cancel
			</Button>
		</ButtonGroup>
	</div>

	{#if showModal}
		<ModalBackdrop>
			<Modal>
				<h1>heading 1</h1>
				<h2>heading 2</h2>
				<h3>heading 3</h3>
				<h4>heading 4</h4>
				<h5>heading 5</h5>
				<p>
					Zombie ipsum reversus ab viral inferno, nam rick grimes malum cerebro. De carne lumbering
					animata corpora quaeritis. Summus brains sit​​, morbo vel maleficia? De apocalypsi gorger
					omero undead survivor dictum mauris. Hi mindless mortuis soulless creaturas, imo evil
					stalking monstra adventus resi dentevil vultus comedat cerebella viventium. Qui animated
					corpse, cricket bat max brucks terribilem incessu zomby. The voodoo sacerdos flesh eater,
					suscitat mortuos comedere carnem virus. Zonbi tattered for solum oculi eorum defunctis go
					lum cerebro. Nescio brains an Undead zombies. Sicut malus putrid voodoo horror. Nigh tofth
					eliv ingdead.
				</p>

				{#snippet footer()}
					<div class="buttons">
						<Button onclick={() => (showModal = false)}>
							<DeleteIcon /> close
						</Button>
					</div>
				{/snippet}
			</Modal>
		</ModalBackdrop>
	{/if}

	<div>
		<h1>heading 1</h1>
		<h2>heading 2</h2>
		<h3>heading 3</h3>
		<h4>heading 4</h4>
		<h5>heading 5</h5>
		<p>
			Zombie ipsum reversus ab viral inferno, nam rick grimes malum cerebro. De carne lumbering
			animata corpora quaeritis. Summus brains sit​​, morbo vel maleficia? De apocalypsi gorger
			omero undead survivor dictum mauris. Hi mindless mortuis soulless creaturas, imo evil stalking
			monstra adventus resi dentevil vultus comedat cerebella viventium. Qui animated corpse,
			cricket bat max brucks terribilem incessu zomby. The voodoo sacerdos flesh eater, suscitat
			mortuos comedere carnem virus. Zonbi tattered for solum oculi eorum defunctis go lum cerebro.
			Nescio brains an Undead zombies. Sicut malus putrid voodoo horror. Nigh tofth eliv ingdead.
		</p>
	</div>
</section>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
	.page .buttons {
		display: flex;
		justify-content: end;
		gap: 0.5em;
	}
	.page .dropdown {
		position: relative;
		margin-left: -1px;
	}
	.page .search {
		display: flex;
		gap: 0.5em;
	}
	.page .search .input {
		flex: 1;
	}
	.page .search .button {
		display: flex;
	}
</style>
