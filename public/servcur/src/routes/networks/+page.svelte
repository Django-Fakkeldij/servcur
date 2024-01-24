<script lang="ts">
	import NetworkCard from '$lib/NetworkCard.svelte';
	import type { Network } from '$lib/docker_types/__generated';
	import { Card, Heading, Secondary, Table, TableBody, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: networks = getContext('data') as Writable<Network[]>;
</script>

<div class="flex items-center justify-center p-4">
	<Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<Table divClass="w-full">
			<caption class="bg-white p-5 text-left text-lg font-semibold text-gray-900 dark:bg-gray-800 dark:text-white">
				<Heading>Networks</Heading>
				<Secondary class="mt-1 text-sm font-normal text-gray-500 dark:text-gray-400">All the existing networks on this machine.</Secondary>
			</caption>
			<TableHead>
				<TableHeadCell>Name</TableHeadCell>
				<TableHeadCell>Created on</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each $networks as network}
					<NetworkCard {network} />
				{/each}
			</TableBody>
		</Table>
	</Card>
</div>
