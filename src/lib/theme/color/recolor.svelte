<script lang="ts">
	import type { Snippet } from 'svelte';
	import { defaultMap, type BrightnessMap } from './mappings';

	let {
		children,
		themeVarPrefix = '--theme-color-primary',
		elementVarPrefix = '--element-color',
		brightnessMap = defaultMap
	}: {
		children?: Snippet;
		themeVarPrefix?: string;
		elementVarPrefix?: string;
		brightnessMap?: BrightnessMap;
	} = $props();

	function createStyle() {
		return Object.entries(brightnessMap)
			.map(([elementBrightness, themeBrightness]) => {
				let elementVar = `${elementVarPrefix}-${elementBrightness}`;
				let themeVar = `${themeVarPrefix}-${themeBrightness}`;
				return `${elementVar}:var(${themeVar})`;
			})
			.join(';');
	}
</script>

<div style={'display:contents;' + createStyle()}>
	{@render children?.()}
</div>
