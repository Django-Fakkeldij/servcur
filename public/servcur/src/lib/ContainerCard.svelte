<script lang="ts">
	import { Badge, Heading, P, Popover, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import { CheckCircleSolid, ClockSolid, CloseCircleSolid, QuestionCircleSolid } from 'flowbite-svelte-icons';
	import type { ContainerSummary } from './docker_types/__generated';
	import { capatalizeWord, dateString } from './util';

	export let container: ContainerSummary;

	export let spacing = false;

	$: name_display = container.Names?.reduce((total: string, current) => total + ' ' + current, '').replaceAll('/', '');
	$: created_at = new Date((container.Created ?? 0) * 1000);

	type IContainerState = { label: string; tailwind_color: string; badge_color: Badge['$$prop_def']['color'] };
	function getContainerState(state: string): IContainerState {
		switch (state) {
			case 'running':
				return {
					tailwind_color: 'text-green-500',
					label: capatalizeWord(state),
					badge_color: 'green',
				};
			case 'exited':
				return {
					tailwind_color: 'text-red-500',
					label: capatalizeWord(state),
					badge_color: 'red',
				};
			default:
				return {
					tailwind_color: 'text-yellow-500',
					label: capatalizeWord(state),
					badge_color: 'yellow',
				};
		}
	}
	$: containerState = getContainerState(container.State ?? '');
</script>

<TableBodyRow>
	<TableBodyCell tdClass={spacing ? 'pl-10' : undefined}>
		<Popover triggeredBy="#containerstate-{container.Id}" class="text-center">
			<Heading tag="h6" color={containerState.tailwind_color}>{containerState.label}</Heading>
			<P>Created on {dateString(created_at)}</P>
		</Popover>
		<div class="flex min-w-min items-center gap-2 {containerState.tailwind_color}">
			{#if container.State === 'exited'}
				<CloseCircleSolid id="containerstate-{container.Id}" />
			{:else if container.State === 'running'}
				<CheckCircleSolid id="containerstate-{container.Id}" />
			{:else}
				<QuestionCircleSolid id="containerstate-{container.Id}" />
			{/if}
		</div>
	</TableBodyCell>
	<TableBodyCell tdClass={spacing ? 'pl-10' : undefined}><Heading tag="h5">{name_display}</Heading></TableBodyCell>
	<TableBodyCell>
		<P>{container.Image}</P>
	</TableBodyCell>
	<TableBodyCell>
		<div class="flex gap-2">
			{#each container.Ports ?? [] as port}
				<Badge large color="indigo">
					{port?.IP ? port.IP + ':' : ''}{port?.PublicPort ? port.PublicPort + ':' : ''}{port?.PrivatePort}{'/' + port?.Type}
				</Badge>
			{/each}
		</div>
	</TableBodyCell>
	<TableBodyCell>
		<Badge color={containerState.badge_color} class="gap-2 text-nowrap p-2">
			<ClockSolid size="sm" />{container.Status}
		</Badge>
	</TableBodyCell>
</TableBodyRow>
