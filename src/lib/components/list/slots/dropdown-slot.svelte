<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children,
		dropdown,
		fill = false,
		position = 'to-right'
	}: {
		children?: Snippet;
		dropdown?: Snippet;
		fill?: Boolean;
		position?: 'to-right' | 'to-left';
	} = $props();
</script>

<div class="slot" class:fill>
	{@render children?.()}

	<div
		class="dropdown-area"
		class:to-left={position === 'to-left'}
		class:to-right={position === 'to-right'}
	>
		{@render dropdown?.()}
	</div>
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
		right: 0;
	}
	.slot .dropdown-area.to-right {
		left: 0;
	}
	.slot.fill {
		flex: 1;
	}
</style>
