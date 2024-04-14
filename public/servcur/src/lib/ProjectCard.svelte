<script lang="ts">
	import { Button, Heading, P, Popover, Spinner, TableBodyCell, TableBodyRow } from 'flowbite-svelte';
	import { CheckCircleSolid, CopySolid } from 'flowbite-svelte-icons';
	import ProjectActions from './ProjectActions.svelte';
	import { API_ROUTES, API_URL } from './api';
	import type { ProjectGet } from './models/projects';
	import { capatalizeWord, makeId } from './util';

	export let project: ProjectGet;
	export let Building: boolean;

	async function onAction(action: 'start' | 'stop' | 'restart') {
		await fetch(API_ROUTES.project_action(project.project_name, project.branch), {
			method: 'POST',
			body: JSON.stringify({ action: action }),
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json',
			},
		}).catch((e) => console.error(e));
	}

	$: webhook_url = `${API_URL}${project.uri}`;
	async function copyWebhook() {
		console.log('h');
		await navigator.clipboard.writeText(webhook_url);
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
		<Heading tag="h5" id={makeId('project', project.project_name)}>
			{project.project_name}
		</Heading>
		<Popover triggeredBy="#{makeId('project', project.project_name)}" class="text-center">
			<div class="flex flex-row items-center gap-2 p-1">
				<Heading tag="h6">Webhook:</Heading><P italic>{webhook_url}</P>
				<Button color="light" class="border-none p-2" on:click={copyWebhook}>
					<CopySolid class="h-5 w-5 outline-none" />
				</Button>
			</div>
		</Popover>
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
		<ProjectActions
			OnStart={async () => await onAction('start')}
			OnStop={async () => await onAction('stop')}
			OnRestart={async () => await onAction('restart')}
		/>
	</TableBodyCell>
</TableBodyRow>
