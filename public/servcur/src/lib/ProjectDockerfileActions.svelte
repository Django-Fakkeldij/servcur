<script lang="ts">
	import { Button, ButtonGroup, Spinner } from 'flowbite-svelte';
	import { API_ROUTES } from './api';
	import type { DockerFileCommands, ProjectActionReturn, ProjectGet } from './models/projects';

	export let project: ProjectGet;

	export let onAction: ((actionTriggered: DockerFileCommands, response: ProjectActionReturn) => Promise<void>) | undefined = undefined;

	async function onDockerFileAction(project: ProjectGet, command: DockerFileCommands) {
		await fetch(API_ROUTES.project_action(project!.project_name, project!.branch), {
			method: 'POST',
			body: JSON.stringify({
				action_kind: {
					project_kind: project.project_kind.type,
					command: command,
				},
			}),
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json',
			},
		})
			.then(async (v) => {
				if (onAction) {
					onAction(command, await v.json());
				}
			})
			.catch((e) => console.error(e));
	}

	const OnStart = async () => await onDockerFileAction(project, 'Start');
	const OnStop = async () => await onDockerFileAction(project, 'Stop');
	const OnBuild = async () => await onDockerFileAction(project, 'Build');

	$: loading = false;
</script>

{#if loading}
	<Spinner />
{:else}
	<ButtonGroup>
		<Button
			class="py-2"
			color="yellow"
			on:click={async () => {
				loading = true;
				await OnBuild();
				loading = false;
			}}>Build</Button
		>
		<Button
			class="py-2"
			color="green"
			on:click={async () => {
				loading = true;
				await OnStart();
				loading = false;
			}}>Start</Button
		>
		<Button
			class="py-2"
			color="red"
			on:click={async () => {
				loading = true;
				await OnStop();
				loading = false;
			}}>Stop</Button
		>
	</ButtonGroup>
{/if}
