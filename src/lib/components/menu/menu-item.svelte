<script lang="ts">
	import { ArrowIcon } from '../icons';
	import { ListItem } from '../list';
	import { AnchorSlot, TextSlot, IconSlot } from '../list/slots';
	import type { Snippet } from 'svelte';

	let {
		children,
		link,
		current
	}: {
		children?: Snippet;
		link?: string;
		current?: boolean;
	} = $props();
</script>

<ListItem>
	<AnchorSlot href={link}>
		<TextSlot fill>
			{@render children?.()}
		</TextSlot>
		<IconSlot>
			<ArrowIcon direction={current ? 'r' : 'l'} animate />
		</IconSlot>
	</AnchorSlot>
</ListItem>
