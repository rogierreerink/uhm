<script lang="ts">
	import { Border } from '$lib/components/boxes';
	import { List, ListItem } from '$lib/components/list';
	import { TextSlot, ButtonSlot, IconSlot, DropdownSlot } from '$lib/components/list/slots';
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
</script>

<section class="page">
	<Border>
		<List>
			{#each items as item, itemIdx}
				<ListItem>
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
				</ListItem>
			{/each}
		</List>
	</Border>

	<div class="add-item">
		<div class="input">
			<Border>
				<TextInput placeholder="add item..." />
			</Border>
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
		gap: 0.5em;
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
