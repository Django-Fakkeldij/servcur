<script lang="ts">
	import { goto } from '$app/navigation';
	import { API_ROUTES } from '$lib/api';
	import { routes } from '$lib/routes';
	import { Button, Heading, Hr, Input, Label, Select } from 'flowbite-svelte';

	let kinds: { value: string; name: string }[] = [{ value: 'DockerFile', name: 'DockerFile' }];
	let project_kind: (typeof kinds)[0]['value'] = kinds[0].value;

	let name: string;
	let https_url: string;
	let auth: string;
	let branch: string;

	async function sub() {
		const body = {
			name: name,
			branch: branch,
			https_url: https_url,
			auth: auth,
			project_kind: {
				type: project_kind,
				image_version: 0,
			},
		};
		console.log(body);
		await fetch(API_ROUTES.project_create, {
			method: 'POST',
			body: JSON.stringify(body),
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json',
			},
		})
			.then(async (v) => {
				if (v.status >= 200 && v.status < 300) goto(routes.projects, { invalidateAll: true });
			})
			.catch((e) => console.error(e));
	}
</script>

<div class="flex flex-col gap-2 p-4">
	<div class="grid grid-cols-2">
		<Heading>New project</Heading>
	</div>
	<Hr hrClass="my-2" />
	<div class="flex max-w-screen-md gap-1">
		<Label class="flex-1">
			Project name
			<Input autofocus bind:value={name} />
		</Label>
		<Label>
			Project kind
			<Select bind:value={project_kind} items={kinds} />
		</Label>
	</div>
	<div class="grid grid-cols-3 gap-2">
		<Label>
			Github (https) url
			<Input class="max-w-80" bind:value={https_url} />
		</Label>
		<Label>
			Github token
			<Input class="max-w-80" type="password" bind:value={auth} />
		</Label>
		<Label>
			Branch
			<Input class="max-w-80" bind:value={branch} />
		</Label>
	</div>
	<Button on:click={() => sub()} class="max-w-fit">Create</Button>
</div>
