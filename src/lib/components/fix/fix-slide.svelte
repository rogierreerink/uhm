<script lang="ts">
	import { onMount, type Snippet } from 'svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();

	let topRef = $state<HTMLElement>();
	let boxRef = $state<HTMLElement>();
	let alreadyFixed = false;

	function slide() {
		if (!topRef || !boxRef) return;

		const relativeScrollTop = window.scrollY - topRef.offsetTop;

		if (relativeScrollTop >= 0 && !alreadyFixed) {
			boxRef.style.position = 'fixed';
			boxRef.style.top = '0px';
			alreadyFixed = true;
		} else if (relativeScrollTop < 0 && alreadyFixed) {
			boxRef.style.position = 'relative';
			boxRef.style.top = 'auto';
			alreadyFixed = false;
		}
	}

	onMount(() => slide());
</script>

<svelte:window onscroll={slide} onresize={slide} />

<!-- Just duplicate the children for the empty space dimensions so we
  -- don't have to deal with subpixel-rendering issues in JavaScript.
  -->
<div bind:this={topRef} class="hidden">
	{@render children?.()}
</div>
<div bind:this={boxRef}>
	{@render children?.()}
</div>

<style>
	.hidden {
		height: 0;
		overflow: hidden;
	}
</style>
