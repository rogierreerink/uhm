<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let props: HTMLInputAttributes = $props();
</script>

<input class="text" type="text" {...props} />

<style>
	.text {
		all: unset;
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding: 0.3em 0.4em;
		color: var(--color-zinc-400);
	}
</style>
