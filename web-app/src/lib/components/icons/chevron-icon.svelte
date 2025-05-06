<script lang="ts">
	import IconInlineRight from '~icons/tabler/chevron-right';

	let {
		direction = 'r',
		animate = false
	}: {
		direction?: 'l' | 'u' | 'd' | 'r';
		animate?: boolean;
	} = $props();
</script>

<span
	class="inline-arrow"
	class:left={direction === 'l'}
	class:up={direction === 'u'}
	class:down={direction === 'd'}
	class:animate
>
	<IconInlineRight />
</span>

<style>
	.inline-arrow {
		display: inline-flex;
	}
	.inline-arrow.left {
		transform: rotate(180deg);
	}
	.inline-arrow.up {
		transform: rotate(-90deg);
	}
	.inline-arrow.down {
		transform: rotate(90deg);
	}
	.inline-arrow.animate {
		transition: ease 0.2s;
	}
</style>
