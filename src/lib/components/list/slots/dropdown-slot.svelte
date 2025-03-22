<script lang="ts">
	import { DropdownBox } from '$lib/components/boxes';
	import { unfoldHeight } from '$lib/transitions';
	import type { Snippet } from 'svelte';
	import ButtonSlot from './button-slot.svelte';

	let {
		children,
		dropdown,
		fill = false,
		position = 'to-right',
		show = false,
		zIndex = 0,
		ontoggle
	}: {
		children?: Snippet;
		dropdown?: Snippet;
		fill?: Boolean;
		position?: 'to-right' | 'to-left';
		show?: Boolean;
		zIndex?: number;
		ontoggle?: () => void;
	} = $props();
</script>

<div class="slot" class:fill>
	<ButtonSlot onclick={() => ontoggle?.()} fill>
		{@render children?.()}
	</ButtonSlot>

	{#if show}
		<div
			class="dropdown-area"
			class:to-left={position === 'to-left'}
			class:to-right={position === 'to-right'}
			style={`z-index:${zIndex}`}
		>
			<DropdownBox>
				<div transition:unfoldHeight>
					{@render dropdown?.()}
				</div>
			</DropdownBox>
		</div>
	{/if}
</div>

<style>
	.slot {
		position: relative;
		display: flex;
		position: relative;
	}
	.slot .dropdown-area {
		display: block;
		position: absolute;
		top: 100%;
	}
	.slot .dropdown-area.to-left {
		right: -1px;
	}
	.slot .dropdown-area.to-right {
		left: -1px;
	}
	.slot.fill {
		flex: 1;
	}
</style>
