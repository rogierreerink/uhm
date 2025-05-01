<script lang="ts">
	import { Box } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import {
		Slot,
		TextSlot,
		ButtonSlot,
		IconSlot,
		DropdownSlot,
		SwipeSlot,
		IconButtonSlot,
		TextButtonSlot
	} from '$lib/components/list/slots';
	import {
		MoreIcon,
		CheckIcon,
		DeleteIcon,
		AddIcon,
		ArrowIcon,
		AtIcon,
		CircleIcon,
		MenuIcon,
		SubstractIcon,
		ChevronIcon
	} from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { CheckInput, TextInput } from '$lib/components/form';
	import { Modal, ModalBackdrop } from '$lib/components/modal';
	import { Label } from '$lib/components/labels';

	const items = $state([
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
	]);

	const options = [{ label: 'options 1' }, { label: 'options 2' }, { label: 'options 3' }];

	let showModal = $state(false);
	let showItemDropdown = $state<number>();
	let showItemSwiped = $state<{
		idx: number;
		area: 'left' | 'right';
		pretriggered?: boolean;
	}>();
</script>

<svelte:head>
	<title>Elements</title>
</svelte:head>

<section class="page">
	<h1>Elements</h1>

	<div class="search">
		<div class="input">
			<Box>
				<div class="stretch">
					<TextInput placeholder="search items..." />
				</div>
			</Box>
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

	<Box>
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
						<DropdownSlot
							show={showItemDropdown === itemIdx}
							zIndex={items.length + 10 - itemIdx}
							ontoggle={() =>
								(showItemDropdown = showItemDropdown !== itemIdx ? itemIdx : undefined)}
						>
							<IconSlot>
								<MoreIcon />
							</IconSlot>

							{#snippet dropdown()}
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
							{/snippet}
						</DropdownSlot>

						<TextSlot fill>
							{item.label}
						</TextSlot>

						<IconButtonSlot onclick={(e) => (item.checked = !item.checked)}>
							<CheckInput checked={item.checked} />
						</IconButtonSlot>

						{#snippet left()}
							<TextButtonSlot>
								{showItemSwiped?.pretriggered ? 'delete !' : 'delete'}
							</TextButtonSlot>
						{/snippet}

						{#snippet right()}
							<TextButtonSlot>
								{showItemSwiped?.pretriggered ? '!' : ''}
								{item.checked ? 'uncheck' : 'check'}
							</TextButtonSlot>
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
	</Box>

	<Box>
		<List>
			{#each options as option, optionIdx}
				<ListItem>
					<TextButtonSlot fill>
						{option.label}
					</TextButtonSlot>
					{#if optionIdx === 0}
						<TextSlot>
							<Label><ArrowIcon /> some label with icon</Label>
						</TextSlot>
						<TextSlot>
							<Label>some label</Label>
						</TextSlot>
					{/if}
				</ListItem>
			{/each}
		</List>
	</Box>

	<Box>
		<List>
			<ListItem>
				<IconSlot>
					<AddIcon />
				</IconSlot>
				<IconSlot>
					<ArrowIcon />
				</IconSlot>
				<IconSlot>
					<AtIcon />
				</IconSlot>
				<IconSlot>
					<CheckIcon />
				</IconSlot>
				<IconSlot>
					<ChevronIcon />
				</IconSlot>
				<IconSlot>
					<CircleIcon />
				</IconSlot>
				<IconSlot>
					<DeleteIcon />
				</IconSlot>
				<IconSlot>
					<MenuIcon />
				</IconSlot>
				<IconSlot>
					<MoreIcon />
				</IconSlot>
				<IconSlot>
					<SubstractIcon />
				</IconSlot>
			</ListItem>
		</List>
	</Box>

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
			Nescio brains an Undead zombies. Sicut <Label>this is a label</Label>
			<Label><ArrowIcon /> this is a label with an icon</Label> incessu zomby. The voodoo sacerdos flesh
			eater, suscitat mortuos comedere carnem virus. Zonbi tattered for solum oculi eorum defunctis go
			lum cerebro.
		</p>
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
</style>
