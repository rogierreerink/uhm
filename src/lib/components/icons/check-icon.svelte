<script lang="ts">
	import IconCheck from '~icons/tabler/check';

	let {
		enable = true
	}: {
		enable?: boolean;
	} = $props();
</script>

<span class="check" class:enable>
	<IconCheck />
</span>

<style>
	.check {
		all: unset;
		display: inline-flex;
		color: var(--element-color-700);
	}
	.check.enable {
		color: var(--element-color-400);
	}
	.check:hover,
	.check:active {
		color: inherit;
	}
</style>
