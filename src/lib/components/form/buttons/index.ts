import { getContext, hasContext, setContext } from 'svelte';

export type GroupContext = {
	orientation: 'horizontal' | 'vertical';
};

export function setGroupContext(ctx: GroupContext) {
	setContext('button-group', ctx);
}

export function getGroupContext() {
	if (hasContext('button-group')) {
		return getContext<GroupContext>('button-group');
	}
}

export { default as Button } from './button.svelte';
export { default as ButtonGroup } from './button-group.svelte';
