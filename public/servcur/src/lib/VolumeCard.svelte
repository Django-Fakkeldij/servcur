<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { Heading, P, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import Actions from './Action.svelte';
	import { API_ROUTES } from './api';
	import type { Volume } from './docker_types/__generated';
	import { dateString } from './util';

	export let volume: Volume;
	$: created_at = new Date(volume.CreatedAt!);

	async function onDelete() {
		await fetch(API_ROUTES.volume_remove(volume.Name), {
			method: 'DELETE',
		}).catch((e) => console.error(e));
		await invalidateAll();
	}
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
	<TableBodyCell>
		<Actions OnAction={onDelete}>X</Actions>
	</TableBodyCell>
</TableBodyRow>
