<script lang="ts">
	import { Box } from '$lib/components/boxes';
	import { CheckIcon } from '$lib/components/icons';
	import { TextInput } from '$lib/components/form';
	import { Button } from '$lib/components/form/buttons';
	import { List, ListItem } from '$lib/components/list';
	import { TextButtonSlot } from '$lib/components/list/slots';

	let {
		placeholder = 'add item...',
		suggestions = [],
		oninput,
		onenter,
		onentersuggestion
	}: {
		placeholder?: string;
		suggestions?: {
			id: string;
			text: string;
		}[];
		oninput?: (value: string) => void;
		onenter?: (value: string) => Promise<boolean>;
		onentersuggestion?: (idx: number) => Promise<boolean>;
	} = $props();

	let value = $state('');
	let suggestionIdx = $state<number>();
</script>

<div class="wrapper">
	<div class="input-section">
		<div class="input-box">
			<Box>
				<div class="input">
					<TextInput
						{placeholder}
						{value}
						oninput={(e) => {
							value = e.currentTarget.value;
							suggestionIdx = undefined;
							oninput?.(value);
						}}
						onkeydown={async (e) => {
							switch (e.key) {
								case 'Enter':
									suggestionIdx = undefined;

									if (await onenter?.(value)) {
										value = '';
									}

									break;

								case 'Tab':
									e.preventDefault();

									if (suggestions.length === 0) {
										break;
									}

									if (e.shiftKey) {
										if (suggestionIdx === undefined || suggestionIdx === 0) {
											suggestionIdx = suggestions.length - 1;
										} else {
											suggestionIdx--;
										}
									} else {
										if (suggestionIdx === undefined || suggestionIdx >= suggestions.length - 1) {
											suggestionIdx = 0;
										} else {
											suggestionIdx++;
										}
									}

									value = suggestions[suggestionIdx].text;
									break;
							}
						}}
					/>
				</div>
			</Box>
		</div>

		<Button
			onclick={async () => {
				if (await onenter?.(value)) {
					value = '';
				}
			}}
		>
			<CheckIcon />
		</Button>
	</div>

	<div class="suggestions">
		{#if suggestions.length > 0}
			<Box>
				<List>
					{#each suggestions as suggestion, idx (suggestion.id)}
						<ListItem>
							<TextButtonSlot
								fill
								onclick={async () => {
									if (await onentersuggestion?.(idx)) {
										value = '';
									}
								}}
							>
								<span
									class:highlighted={suggestionIdx === idx}
									class:dimmed={suggestionIdx !== undefined && suggestionIdx !== idx}
								>
									{suggestion.text}
								</span>
							</TextButtonSlot>
						</ListItem>
					{/each}
				</List>
			</Box>
		{/if}
	</div>
</div>

<style>
	.wrapper {
		position: relative;
	}
	.wrapper .input-section {
		display: flex;
		gap: 0.5em;
	}
	.wrapper .input-section .input-box {
		flex: 1;
	}
	.wrapper .input-section .input-box .input {
		display: flex;
	}
	.wrapper .suggestions {
		position: absolute;
		top: calc(100% + 0.5em);
		width: 100%;
	}
	.wrapper .suggestions:hover,
	.wrapper .suggestions .dimmed {
		color: var(--theme-color-primary-400);
	}
	.wrapper .suggestions:not(:hover) .highlighted,
	.wrapper .suggestions span:hover {
		color: white;
	}
</style>
