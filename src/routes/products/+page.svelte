<script lang="ts">
	import { Border } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import { Slot, TextSlot, ButtonSlot, IconSlot } from '$lib/components/list/slots';
	import { MoreIcon, CheckIcon, DeleteIcon } from '$lib/components/icons';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { TextInput } from '$lib/components/form';

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
</script>

<section class="page">
	<div class="search">
		<div class="input">
			<Border>
				<TextInput placeholder="search products..." />
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
			{#each items as item}
				<ListItem>
					<ButtonSlot>
						<IconSlot>
							<MoreIcon />
						</IconSlot>
					</ButtonSlot>

					<TextSlot fill>
						{item.label}
					</TextSlot>

					<ButtonSlot>
						<IconSlot>
							<CheckIcon enable={item.checked} />
						</IconSlot>
					</ButtonSlot>
				</ListItem>
			{/each}

			<ListItem>
				<Slot fill>
					<TextInput placeholder="add item" />
				</Slot>
			</ListItem>
		</List>
	</Border>
</section>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
	.search {
		display: flex;
		gap: 0.5em;
	}
	.search .input {
		flex: 1;
	}
	.search .button {
		display: flex;
	}
</style>
