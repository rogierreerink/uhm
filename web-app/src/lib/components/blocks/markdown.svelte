<script lang="ts" module>
	export type MarkdownData = {
		markdown: string;
		html: string;
	};
</script>

<script lang="ts">
	let {
		data
	}: {
		data: MarkdownData;
	} = $props();
</script>

<div>
	{@html data.html}
</div>
