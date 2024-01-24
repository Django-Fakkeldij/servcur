<script lang="ts">
	import { Heading, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import type { ImageSummary } from './docker_types/__generated';
	import { dateString, fileSizeMagnitudeBytes } from './util';

	export let image: ImageSummary;

	$: name_display = image.RepoTags?.reduce((total: string, current) => total + ' ' + current, '');
	$: created_at = new Date((image.Created ?? 0) * 1000);
	$: fileSize = fileSizeMagnitudeBytes(image.Size);
</script>

<TableBodyRow>
	<TableBodyCell>
		<Heading tag="h5" id="imageName">
			{name_display.length === 0 ? '<none>' : name_display}
		</Heading>
	</TableBodyCell>
	<TableBodyCell>
		{dateString(created_at)}
	</TableBodyCell>
	<TableBodyCell>
		{fileSize[0].toFixed(2)}
		{fileSize[1]}
	</TableBodyCell>
</TableBodyRow>
