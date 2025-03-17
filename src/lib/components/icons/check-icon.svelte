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

		color: var(--color, var(--color-zinc-700));
	}
	.check.enable {
		color: var(--color-checked, var(--color-zinc-400));
	}
	.check:hover {
		color: inherit;
	}
</style>
