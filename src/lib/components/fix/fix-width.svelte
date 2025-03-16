<script lang="ts">
	import { type Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();
</script>

<!-- Just duplicate the children for the empty space dimensions so we
  -- don't have to deal with subpixel-rendering issues in JavaScript.
  -->
<div class="hidden">
	{@render children?.()}
</div>
<div class="fixed">
	{@render children?.()}
</div>

<style>
	.fixed {
		position: fixed;
	}
	.hidden {
		height: 0;
		overflow: hidden;
	}
</style>
