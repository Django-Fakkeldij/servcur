<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { Button, Heading, P, Popover, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import { API_ROUTES } from './api';
	import type { ImageSummary } from './docker_types/__generated';
	import { dateString, fileSizeMagnitudeBytes } from './util';

	export let image: ImageSummary;

	$: id = image.Id.replace('sha256:', '');
	$: name_display = image.RepoTags?.reduce((total: string, current) => total + ' ' + current, '');
	$: created_at = new Date((image.Created ?? 0) * 1000);
	$: fileSize = fileSizeMagnitudeBytes(image.Size);
	let clicked = 0;
	async function checkclicked(c: number) {
		if (c >= 2) {
			clicked = 0;
			if (!image.RepoTags.at(0)) {
				console.error('No tag to delete');
				return;
			}
			await fetch(API_ROUTES.image_remove(image.RepoTags[0]), {
				method: 'DELETE',
			}).catch((e) => console.error(e));
			await invalidateAll();
		}
	}
	$: checkclicked(clicked);
</script>

<TableBodyRow>
	<TableBodyCell>
		<Heading tag="h5" id="imageName-{id}">
			{name_display.length === 0 ? '<none>' : name_display}
		</Heading>
		<Popover triggeredBy="#imageName-{id}" class="text-center">
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
		<Button class="py-2" color={clicked === 1 ? 'red' : 'primary'} on:click={() => (clicked += 1)}>X</Button>
	</TableBodyCell>
</TableBodyRow>
