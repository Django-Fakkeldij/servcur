<script lang="ts">
	import { Heading, P, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import type { Volume } from './docker_types/__generated';
	import { dateString } from './util';

	export let volume: Volume;
	$: created_at = new Date(volume.CreatedAt!);
</script>

<TableBodyRow>
	<TableBodyCell>
		<Heading tag="h5" id="imageName">
			{volume.Name}
		</Heading>
	</TableBodyCell>
	<TableBodyCell>
		<P>{volume.Mountpoint}</P>
	</TableBodyCell>
	<TableBodyCell>
		{dateString(created_at)}
	</TableBodyCell>
</TableBodyRow>
