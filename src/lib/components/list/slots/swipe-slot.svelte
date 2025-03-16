<script lang="ts">
	import { unfoldWidth } from '$lib/transitions';
	import { onMount, type Snippet } from 'svelte';

	let {
		children,
		left,
		right,
		show,
		onshow,
		onpretrigger,
		onpretriggerrevert,
		ontrigger,
		onclose
	}: {
		children?: Snippet;
		left?: Snippet;
		right?: Snippet;
		show?: 'left' | 'right';
		onshow?: (area: 'left' | 'right') => void;
		onpretrigger?: () => void;
		onpretriggerrevert?: () => void;
		ontrigger?: () => void;
		onclose?: () => void;
	} = $props();

	let containerRef = $state<HTMLElement>();
	let containerEmSizePx = 16;

	let swipeTouches = $state<number[]>([]);
	let swipeOffsetEm = $state<number>(0);
	let swipeTrigger = $state(false);

	const minSwipeDistanceEm = 3;
	const minTriggerDistanceEm = 6;

	function registerSwipeStart(e: TouchEvent) {
		if (show === 'left') swipeOffsetEm = minSwipeDistanceEm;
		else if (show === 'right') swipeOffsetEm = -minSwipeDistanceEm;
		registerSwipe(e);
	}

	function registerSwipe(e: TouchEvent) {
		if (e.touches.length !== 1) return;
		const touch = e.touches[0];
		swipeTouches.push(touch.clientX);

		if (swipeTouches.length < 2) return;
		const swipeDistancePx = swipeTouches[swipeTouches.length - 1] - swipeTouches[0];
		const swipeDistanceEm = swipeDistancePx / containerEmSizePx + swipeOffsetEm;
		const triggerDistanceEm = Math.abs(swipeDistancePx / containerEmSizePx);

		if (!swipeTrigger && triggerDistanceEm >= minTriggerDistanceEm) {
			onpretrigger?.();
			swipeTrigger = true;
		} else if (swipeTrigger && triggerDistanceEm <= minTriggerDistanceEm) {
			onpretriggerrevert?.();
			swipeTrigger = false;
		}

		if (swipeDistanceEm >= minSwipeDistanceEm) {
			if (show === 'right') {
				onclose?.();
			} else if (show !== 'left') {
				onshow?.('left');
			}
		} else if (swipeDistanceEm <= -minSwipeDistanceEm) {
			if (show === 'left') {
				onclose?.();
			} else if (show !== 'right') {
				onshow?.('right');
			}
		} else if (
			(show === 'right' && swipeDistanceEm <= 0) ||
			(show === 'left' && swipeDistanceEm >= 0)
		) {
			onclose?.();
		}
	}

	function exitSwipe() {
		if (swipeTrigger) {
			ontrigger?.();
		}

		swipeTouches = [];
		swipeTrigger = false;
	}

	onMount(() => {
		if (containerRef) {
			containerEmSizePx = parseFloat(getComputedStyle(containerRef).fontSize);
		}
	});
</script>

<div
	class="swipe"
	ontouchstart={registerSwipeStart}
	ontouchmove={registerSwipe}
	ontouchend={exitSwipe}
	ontouchcancel={exitSwipe}
	bind:this={containerRef}
>
	{#if show === 'left'}
		<div class="left-area" transition:unfoldWidth>
			{@render left?.()}
		</div>
	{/if}

	<div class="main">
		{@render children?.()}
	</div>

	{#if show === 'right'}
		<div class="right-area" transition:unfoldWidth>
			{@render right?.()}
		</div>
	{/if}
</div>

<style>
	.swipe {
		flex: 1;
		display: flex;
		touch-action: none;
		user-select: none;
	}
	.swipe .main {
		flex: 1;
		display: flex;
	}
	.swipe .left-area,
	.swipe .right-area {
		display: flex;
		overflow: hidden;
		transition: ease 0.2s;
	}
</style>
