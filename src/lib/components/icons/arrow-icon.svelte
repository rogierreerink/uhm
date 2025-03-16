<script lang="ts">
	import IconArrowRight from '~icons/quill/arrow-right';

	let {
		direction = 'r',
		animate = false
	}: {
		direction?: 'l' | 'u' | 'd' | 'r';
		animate?: boolean;
	} = $props();
</script>

<div
	class="arrow"
	class:left={direction === 'l'}
	class:up={direction === 'u'}
	class:down={direction === 'd'}
	class:animate
>
	<IconArrowRight />
</div>

<style>
	.arrow {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
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
