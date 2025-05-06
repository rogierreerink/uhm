<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import ButtonSlot from './button-slot.svelte';
	import IconSlot from './icon-slot.svelte';

	let props: {
		children?: Snippet;
	} & HTMLButtonAttributes = $props();
</script>

<ButtonSlot {...props}>
	<IconSlot>
		{@render props.children?.()}
	</IconSlot>
</ButtonSlot>
