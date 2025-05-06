<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLAnchorAttributes } from 'svelte/elements';
	import TextSlot from './text-slot.svelte';
	import AnchorSlot from './anchor-slot.svelte';

	let props: {
		children?: Snippet;
		fill?: Boolean;
	} & HTMLAnchorAttributes = $props();
</script>

<AnchorSlot {...props}>
	<TextSlot fill>
		{@render props.children?.()}
	</TextSlot>
</AnchorSlot>
