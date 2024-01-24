<script lang="ts">
	import { Heading, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import type { Network } from './docker_types/__generated';
	import { dateString } from './util';

	export let network: Network;
	$: created_at = new Date(network.Created!);
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
</TableBodyRow>
