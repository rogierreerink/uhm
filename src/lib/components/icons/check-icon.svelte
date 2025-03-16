<script lang="ts">
	import IconCheckMark from '~icons/quill/checkmark';

	let {
		enable = true
	}: {
		enable?: boolean;
	} = $props();
</script>

<div class="check" class:enable>
	<IconCheckMark />
</div>

<style>
	.check {
		all: unset;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		color: var(--color, var(--color-zinc-700));
	}
	.check.enable {
		color: var(--color-checked, var(--color-zinc-400));
	}
	.check:hover {
		color: inherit;
	}
</style>
