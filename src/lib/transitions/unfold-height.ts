import { sineInOut } from 'svelte/easing';

export function unfoldHeight(
	node: HTMLElement,
	params?: { delay?: number; duration?: number; easing?: (t: number) => number; zIndex?: number }
) {
	return {
		delay: params?.delay || 0,
		duration: params?.duration || 200,
		easing: params?.easing || sineInOut,
		css: (t: number) => (t === 1 ? '' : `height:${t * node.clientHeight}px; overflow:hidden;`)
	};
}
