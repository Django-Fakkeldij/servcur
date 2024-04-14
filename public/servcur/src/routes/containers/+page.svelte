<script lang="ts">
	import ContainerCard from '$lib/ContainerCard.svelte';
	import StackCard from '$lib/StackCard.svelte';
	import type { ContainerSummary } from '$lib/docker_types/__generated';
	import { A, Card, Heading, P, Popover, Secondary, Table, TableBody, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: containers = getContext('data') as Writable<ContainerSummary[]>;

	type IStackGrouping = Map<string, ContainerSummary[]>;

	const stackKey = 'com.docker.compose.project';
	function mapContainers(containers: ContainerSummary[]): [ContainerSummary[], ContainerSummary[][]] {
		const no_stack: ContainerSummary[] = containers.filter((container) => !(stackKey in (container?.Labels ?? {})));

		const stacks: IStackGrouping = new Map();

		containers.forEach((container) => {
			if (container?.Labels && stackKey in container.Labels) {
				const old = stacks.get(container.Labels[stackKey]) ?? [];
				stacks.set(container.Labels[stackKey], [...old, container]);
			}
		});

		return [no_stack, Array.from(stacks.values())];
	}
	$: [containers_no_stack, stacks] = mapContainers($containers);
</script>

<div class="flex items-center justify-center p-4">
	<Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<Table divClass="w-full">
			<caption class="bg-white p-5 text-left text-lg font-semibold text-gray-900 dark:bg-gray-800 dark:text-white">
				<Heading>Containers</Heading>
				<Secondary class="mt-1 text-sm font-normal text-gray-500 dark:text-gray-400">
					All the existing containers on this machine.
				</Secondary>
			</caption>
			<Popover triggeredBy="#portSyntax" class="text-center">
				<Heading tag="h6">Port syntax</Heading>
				<P>[ IP? ] : [ PublicPort? ] : [ PrivatePort ] / [protocol]</P>
				<A href="https://docs.docker.com/network/">Docs</A>
			</Popover>
			<TableHead>
				<TableHeadCell></TableHeadCell>
				<TableHeadCell>Name</TableHeadCell>
				<TableHeadCell>Image</TableHeadCell>
				<TableHeadCell id="portSyntax">Ports</TableHeadCell>
				<TableHeadCell>Status</TableHeadCell>
				<TableHeadCell>Actions</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each containers_no_stack as container}
					<ContainerCard {container} />
				{/each}
				{#each stacks as stack}
					<StackCard containers={stack} />
				{/each}
			</TableBody>
		</Table>
	</Card>
</div>
