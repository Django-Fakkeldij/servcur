<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { Heading, P, Popover, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import Actions from './Actions.svelte';
	import { API_ROUTES } from './api';
	import type { ImageSummary } from './docker_types/__generated';
	import { dateString, fileSizeMagnitudeBytes, makeId } from './util';

	export let image: ImageSummary;

	$: id = image.Id.replace('sha256:', '');
	$: name_display = image.RepoTags?.reduce((total: string, current) => total + ' ' + current, '');
	$: created_at = new Date((image.Created ?? 0) * 1000);
	$: fileSize = fileSizeMagnitudeBytes(image.Size);

	$: visible = true;
	async function onDelete() {
		let n = image.RepoTags.length !== 0 ? image.RepoTags[0] : image.Id;
		let ret = await fetch(API_ROUTES.image_remove(n), {
			method: 'DELETE',
		})
			.then((v) => v.text())
			.catch((e) => {
				console.error(e);
			});
		await invalidateAll();
		if (ret && ret.includes('Untagged')) {
			visible = true;
			alert('Image was just untagged and not deleted.');
		} else {
			visible = false;
		}
	}
</script>

<TableBodyRow class={visible ? 'visible' : 'hidden'}>
	<TableBodyCell>
		<Heading tag="h5" id={makeId('imageName', id)}>
			{name_display.length === 0 ? '<none>' : name_display}
		</Heading>
		<Popover triggeredBy="#{makeId('imageName', id)}" class="text-center">
			<Heading tag="h6">ID:</Heading>
			<P italic>{id}</P>
		</Popover>
	</TableBodyCell>
	<TableBodyCell>
		{dateString(created_at)}
	</TableBodyCell>
	<TableBodyCell>
		{fileSize[0].toFixed(2)}
		{fileSize[1]}
	</TableBodyCell>
	<TableBodyCell>
		<Actions OnDelete={onDelete} />
	</TableBodyCell>
</TableBodyRow>
