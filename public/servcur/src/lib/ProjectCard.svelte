<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { A, Heading, P, Spinner, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import { CheckCircleSolid } from 'flowbite-svelte-icons';
	import Action from './Action.svelte';
	import { API_ROUTES } from './api';
	import type { ProjectGet } from './models/projects';
	import { routes } from './routes';
	import { capatalizeWord } from './util';

	export let project: ProjectGet;
	export let Building: boolean;

	async function onDelete() {
		await fetch(API_ROUTES.project_remove(project.project_name, project.branch), {
			method: 'DELETE',
		}).catch((e) => console.error(e));
		await invalidateAll();
	}
</script>

<TableBodyRow>
	<TableBodyCell>
		{#if !Building}
			<CheckCircleSolid />
		{:else}
			<Spinner />
		{/if}
	</TableBodyCell>
	<TableBodyCell>
		<A href={routes.project(project.project_name, project.branch)}>
			<Heading tag="h5">
				{project.project_name}
			</Heading>
		</A>
	</TableBodyCell>
	<TableBodyCell>
		{project.branch}
	</TableBodyCell>
	<TableBodyCell>
		<P>{capatalizeWord(project.project_kind.type)}</P>
	</TableBodyCell>
	<TableBodyCell>
		{project.path}
	</TableBodyCell>
	<TableBodyCell>
		<Action OnAction={onDelete}>X</Action>
	</TableBodyCell>
</TableBodyRow>
