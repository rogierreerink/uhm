<script lang="ts">
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { Breadcrumb, BreadcrumbTrail } from '$lib/components/breadcrumb';
	import { Label, TextInput } from '$lib/components/form';
	import { Button, ButtonGroup } from '$lib/components/form/buttons';
	import { CheckIcon, DeleteIcon, UndoIcon } from '$lib/components/icons';
	import { product, type ProductResponse } from '$lib/data/products';

	let {
		data
	}: {
		data: ProductResponse;
	} = $props();

	let name = $state(data.data.name);
	let hasChanged = $derived(name !== data.data.name);

	async function saveChanges() {
		await product.patch(page.params.id, { name });
		await invalidate(product.url(page.params.id));
	}

	async function revertChanges() {
		name = data.data.name;
	}

	async function deleteProduct() {
		await product.delete(page.params.id);
		goto('.', {
			replaceState: true
		});
	}
</script>

<section class="page">
	<h1>{data.data.name}</h1>

	<BreadcrumbTrail>
		<Breadcrumb href=".">products</Breadcrumb>
		<Breadcrumb>{data.data.name.toLocaleLowerCase()}</Breadcrumb>
	</BreadcrumbTrail>

	<div class="data">
		<div class="input">
			<Label for="name">Name</Label>
			<TextInput
				id="name"
				placeholder="name"
				value={name}
				oninput={(e) => (name = e.currentTarget.value.trim())}
			/>
		</div>
	</div>

	<div class="buttons">
		<ButtonGroup orientation="horizontal">
			<Button onclick={() => saveChanges()} disabled={!hasChanged}>
				<CheckIcon /> save
			</Button>
			<Button onclick={() => revertChanges()} disabled={!hasChanged}>
				<UndoIcon /> revert
			</Button>
			<Button
				onclick={() => deleteProduct()}
				disabled={data.data.shopping_list_item_links.length > 0}
			>
				<DeleteIcon /> delete
			</Button>
		</ButtonGroup>
	</div>

	<div class="metadata">
		<div>Created: {data.created.toUTCString()}</div>
		<div>Updated: {data.updated?.toUTCString() ?? 'never'}</div>
	</div>
</section>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1em;
	}
	.page .data .input {
		display: flex;
	}
	.page .metadata {
		display: block;
		font-size: 0.8em;
		font-style: italic;
		text-align: right;
		margin: 0.3em 0.4em;
	}
	.page .buttons {
		display: flex;
		justify-content: end;
		gap: 1em;
	}
</style>
