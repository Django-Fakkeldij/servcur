<script lang="ts">
	import ContainerCard from '$lib/ContainerCard.svelte';
	import { Badge, Button, Heading, P, Popover, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import { ChevronDownSolid, ChevronRightSolid, ClockSolid, LayersSolid } from 'flowbite-svelte-icons';
	import type { ContainerSummary } from './docker_types/__generated';
	import { capatalizeWord } from './util';

	export let containers: ContainerSummary[];

	const matchKey = 'com.docker.compose.project';

	$: name = containers.map((val) => val?.Labels).at(0)![matchKey];

	enum EContainerStatus {
		running,
		exited,
		unknown,
	}
	type IContainerState = { status: EContainerStatus; label: string; tailwind_color: string; badge_color: Badge['$$prop_def']['color'] };
	function getStackState(state: string): IContainerState {
		switch (state) {
			case 'running':
				return {
					status: EContainerStatus.running,
					tailwind_color: 'text-green-500',
					label: capatalizeWord(state),
					badge_color: 'green',
				};
			case 'exited':
				return {
					status: EContainerStatus.exited,
					tailwind_color: 'text-red-500',
					label: capatalizeWord(state),
					badge_color: 'red',
				};
			default:
				return {
					status: EContainerStatus.unknown,
					tailwind_color: 'text-yellow-500',
					label: capatalizeWord(state),
					badge_color: 'yellow',
				};
		}
	}
	$: containerStates = containers.map((container) => getStackState(container.State ?? ''));
	$: runningContainers = containerStates.filter((state) => state.status === EContainerStatus.running).length;
	$: notRunningContainers = containers.length - runningContainers;
	$: collapsedState = (
		notRunningContainers === 0
			? {
					status: EContainerStatus.running,
					tailwind_color: 'text-green-500',
					label: capatalizeWord('running'),
					badge_color: 'green',
				}
			: {
					status: EContainerStatus.exited,
					tailwind_color: 'text-red-500',
					label: capatalizeWord('exited'),
					badge_color: 'red',
				}
	) as IContainerState;

	$: created_at_raw = containers.reduce(
		(highest, current) => ((current?.Created ?? 0) > highest!.Created! ? current : highest),
		containers.at(0)
	);
	$: created_at = new Date((created_at_raw?.Created ?? 0) * 1000);

	$: open = false;
</script>

<TableBodyRow>
	<TableBodyCell>
		<div class="flex items-center justify-between gap-2">
			<Popover triggeredBy="#containerstate-{name}" class="text-center">
				<Heading tag="h6" color={collapsedState.tailwind_color}>{collapsedState.label} ({containers.length}/{runningContainers})</Heading>
				<P>Created on {created_at.toLocaleDateString()} at {created_at.toLocaleTimeString()}</P>
			</Popover>
			<LayersSolid id="containerstate-{name}" class="h-5 w-5 {collapsedState.tailwind_color}" />
			{#if open}
				<Button color="alternative" on:click={() => (open = !open)}>
					<ChevronDownSolid size="xs" />
				</Button>
			{:else}
				<Button color="alternative" on:click={() => (open = !open)}>
					<ChevronRightSolid size="xs" />
				</Button>
			{/if}
		</div>
	</TableBodyCell>
	<TableBodyCell><Heading tag="h5">{name}</Heading></TableBodyCell>
	<TableBodyCell></TableBodyCell>
	<TableBodyCell></TableBodyCell>
	<TableBodyCell>
		<Badge color={collapsedState.badge_color} class="gap-2 text-nowrap p-2">
			<ClockSolid size="sm" />{created_at_raw?.Status}
		</Badge>
	</TableBodyCell>
</TableBodyRow>
{#if open}
	{#each containers as container}
		<ContainerCard {container} spacing />
	{/each}
{/if}
