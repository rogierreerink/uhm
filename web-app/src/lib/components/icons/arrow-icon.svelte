<script lang="ts">
	import IconArrowRight from '~icons/tabler/arrow-narrow-right';

	let {
		direction = 'r',
		animate = false
	}: {
		direction?: 'l' | 'u' | 'd' | 'r';
		animate?: boolean;
	} = $props();
</script>

<span
	class="arrow"
	class:left={direction === 'l'}
	class:up={direction === 'u'}
	class:down={direction === 'd'}
	class:animate
>
	<IconArrowRight />
</span>

<style>
	.arrow {
		display: inline-flex;
	}
	.arrow.left {
		transform: rotate(180deg);
	}
	.arrow.up {
		transform: rotate(-90deg);
	}
	.arrow.down {
		transform: rotate(90deg);
	}
	.arrow.animate {
		transition: ease 0.2s;
	}
</style>
