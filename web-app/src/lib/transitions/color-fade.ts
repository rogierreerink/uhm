import { sineInOut } from 'svelte/easing';

export function colorFade(
	node: HTMLElement,
	params?: {
		color?: { start?: string; end?: string; color_space?: string };
		delay?: number;
		duration?: number;
		easing?: (t: number) => number;
		zIndex?: number;
	}
) {
	const color_start = params?.color?.start ?? 'red';
	const color_end = params?.color?.end ?? 'white';
	const color_space = params?.color?.color_space ?? 'oklab';
	return {
		delay: params?.delay || 0,
		duration: params?.duration || 1000,
		easing: params?.easing || sineInOut,
		css: (t: number) =>
			`color:color-mix(in ${color_space}, ${color_start}, ${color_end} ${t * 100}%)`
	};
}
