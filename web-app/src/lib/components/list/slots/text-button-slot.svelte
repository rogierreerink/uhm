<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import ButtonSlot from './button-slot.svelte';
	import TextSlot from './text-slot.svelte';

	let props: {
		children?: Snippet;
		fill?: Boolean;
	} & HTMLButtonAttributes = $props();
</script>

<ButtonSlot {...props}>
	<TextSlot fill>
		{@render props.children?.()}
	</TextSlot>
</ButtonSlot>
