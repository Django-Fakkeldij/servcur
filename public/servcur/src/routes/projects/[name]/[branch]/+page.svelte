<script lang="ts">
	import ConsoleViewer from '$lib/ConsoleViewer.svelte';
	import ProjectDockerfileActions from '$lib/ProjectDockerfileActions.svelte';
	import { API_ROUTES, API_URL } from '$lib/api.js';
	import type { ProjectGet } from '$lib/models/projects.js';
	import { Card, Heading, Hr, P } from 'flowbite-svelte';
	import { getContext, onDestroy } from 'svelte';
	import type { Writable } from 'svelte/store';

	export let data;

	const projects = getContext('data') as Writable<ProjectGet[]>;

	$: project = $projects.find((v) => data.name === v.project_name && data.branch === v.branch);

	const active_conns = [] as WebSocket[];
	const channel = new MessageChannel();
	channel.port1.start();

	function ConnectToIo(id: string) {
		const ws_stdout = new WebSocket(API_ROUTES.project_io_ws(id, 'stdout'));
		ws_stdout.onmessage = (ev) => {
			channel.port1.postMessage(ev.data);
		};
		const ws_stderr = new WebSocket(API_ROUTES.project_io_ws(id, 'stderr'));
		ws_stderr.onmessage = (ev) => {
			channel.port1.postMessage(ev.data);
		};
		active_conns.push(ws_stderr);
		active_conns.push(ws_stdout);
	}

	onDestroy(() => {
		active_conns.forEach((c) => c.close());
	});
</script>

<div class="flex items-center justify-center p-4">
	<Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<div class="flex max-h-[80vh] w-full flex-col gap-4">
			<div class="flex">
				<Heading>{project?.project_name} ({project?.branch})</Heading>
				{#if project?.project_kind.type === 'DockerFile'}
					<ProjectDockerfileActions
						{project}
						onAction={async (_, res) => {
							ConnectToIo(res.io_id);
						}}
					/>
				{/if}
			</div>
			<Hr hrClass="my-1" />
			<div class="flex flex-col">
				<div class="grid grid-cols-2">
					<div class="flex flex-col">
						<Heading tag="h6">Project kind:</Heading>
						<P>{project?.project_kind.type}</P>
					</div>
					<div class="flex flex-col">
						<Heading tag="h6">Webhook:</Heading>
						<P>{API_URL}{project?.uri}</P>
					</div>
					<div class="flex flex-col">
						<Heading tag="h6">Path on disk:</Heading>
						<P>{project?.path}</P>
					</div>
				</div>
				<Hr hrClass="my-4" />
				<div class="h-72">
					<ConsoleViewer port={channel.port2} />
				</div>
			</div>
		</div>
	</Card>
</div>
