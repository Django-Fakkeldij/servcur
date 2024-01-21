<script lang="ts">
	import { Badge, Heading, P, Popover, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import { CheckCircleSolid, ClockSolid, CloseCircleSolid, QuestionCircleSolid } from 'flowbite-svelte-icons';
	import type { ContainerSummary } from './docker_types/__generated';
	import { capatalizeWord } from './util';

	export let container: ContainerSummary;

	$: name_display = container.Names?.reduce((total: string, current) => total + ' ' + current, '').replaceAll('/', '');
	$: created_at = new Date((container.Created ?? 0) * 1000);

	type IContainerState = { label: string; tailwind_color: string; badge_color: Badge['$$prop_def']['color'] };
	function getContainerState(state: string): IContainerState {
		switch (state) {
			case 'running':
				return {
					tailwind_color: 'text-green-500',
					label: capatalizeWord(container.State),
					badge_color: 'green',
				};
			case 'exited':
				return {
					tailwind_color: 'text-red-500',
					label: capatalizeWord(container.State),
					badge_color: 'red',
				};
			default:
				return {
					tailwind_color: 'text-yellow-500',
					label: capatalizeWord(container.State),
					badge_color: 'yellow',
				};
		}
	}
	$: containerState = getContainerState(container.State ?? '');
</script>

<TableBodyRow>
	<TableBodyCell>
		<Popover triggeredBy="#containerstate" class="text-center">
			<Heading tag="h6" color={containerState.tailwind_color}>{containerState.label}</Heading>
			<P>Created on {created_at.toLocaleDateString()} at {created_at.toLocaleTimeString()}</P>
		</Popover>
		<div class="flex min-w-min items-center gap-2 {containerState.tailwind_color}">
			{#if container.State === 'exited'}
				<CloseCircleSolid id="containerstate" />
			{:else if container.State === 'running'}
				<CheckCircleSolid id="containerstate" />
			{:else}
				<QuestionCircleSolid id="containerstate" />
			{/if}
		</div>
	</TableBodyCell>
	<TableBodyCell><Heading tag="h5">{name_display}</Heading></TableBodyCell>
	<TableBodyCell>
		<P>{container.Image}</P>
	</TableBodyCell>
	<TableBodyCell>
		<Badge color={containerState.badge_color} class="gap-2 text-nowrap p-2">
			<ClockSolid size="sm" />{container.Status}
		</Badge>
	</TableBodyCell>
</TableBodyRow>
