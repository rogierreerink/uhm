<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { CheckIcon, PointIcon } from '../icons';

	let props: HTMLInputAttributes = $props();
</script>

<div class="checkbox">
	<input type="checkbox" {...props} />
	<div class="checked-icon">
		<CheckIcon />
	</div>
	<div class="unchecked-icon">
		<PointIcon />
	</div>
</div>

<style>
	.checkbox {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		flex: 1;
	}
	.checkbox input {
		all: unset;
		position: absolute;
		inset: 0;
		cursor: pointer;
	}
	.checkbox .checked-icon,
	.checkbox .unchecked-icon {
		display: none;
	}
	.checkbox:has(input:checked) .checked-icon,
	.checkbox:not(:has(input:checked)) .unchecked-icon {
		display: flex;
	}
</style>
