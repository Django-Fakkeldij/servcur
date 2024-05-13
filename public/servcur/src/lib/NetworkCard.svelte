<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { Heading, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import Actions from './Action.svelte';
	import { API_ROUTES } from './api';
	import type { Network } from './docker_types/__generated';
	import { dateString } from './util';

	export let network: Network;
	$: created_at = new Date(network.Created!);

	async function onDelete() {
		await fetch(API_ROUTES.network_remove(network.Name!), {
			method: 'DELETE',
		}).catch((e) => console.error(e));
		await invalidateAll();
	}
</script>

<TableBodyRow>
	<TableBodyCell>
		<Heading tag="h5" id="imageName">
			{network.Name}
		</Heading>
	</TableBodyCell>
	<TableBodyCell>
		{dateString(created_at)}
	</TableBodyCell>
	<TableBodyCell>
		<Actions OnAction={onDelete}>X</Actions>
	</TableBodyCell>
</TableBodyRow>
