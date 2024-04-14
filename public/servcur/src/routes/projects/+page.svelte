<script lang="ts">
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: data = getContext('data') as Writable<any>;
</script>

<div class="flex items-center justify-center p-4">
	<!-- <Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<Table divClass="w-full">
			<caption class="bg-white p-5 text-left text-lg font-semibold text-gray-900 dark:bg-gray-800 dark:text-white">
				<Heading>Volumes</Heading>
				<Secondary class="mt-1 text-sm font-normal text-gray-500 dark:text-gray-400">All the existing volumes on this machine.</Secondary>
			</caption>
			<TableHead>
				<TableHeadCell>Name</TableHeadCell>
				<TableHeadCell>Located at</TableHeadCell>
				<TableHeadCell>Created on</TableHeadCell>
				<TableHeadCell>Actions</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each $volumes.Volumes as volume}
					<VolumeCard {volume} />
				{/each}
			</TableBody>
		</Table>
	</Card> -->

	{JSON.stringify(data, null, 4)}
</div>
