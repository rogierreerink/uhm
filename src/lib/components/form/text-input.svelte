<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let props: HTMLInputAttributes = $props();
</script>

<input
	class="text"
	type="text"
	autocomplete="off"
	data-1p-ignore
	data-lpignore="true"
	data-protonpass-ignore="true"
	{...props}
/>

<style>
	.text {
		all: unset;
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: center;
		padding: 0.3em 0.4em;
	}
</style>
