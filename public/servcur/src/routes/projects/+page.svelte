<script lang="ts">
	import ProjectCard from '$lib/ProjectCard.svelte';
	import { API_ROUTES } from '$lib/api';
	import type { ProjectGet } from '$lib/models/projects';
	import { routes } from '$lib/routes';
	import { A, Card, Heading, Secondary, Table, TableBody, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: projects = getContext('data') as Writable<ProjectGet[]>;

	$: build_status = new Map<string, { name: string; branch: string }>();
	$: projects_building = [] as { name: string; branch: string }[];
	async function retrieveBuildstatus() {
		build_status = new Map(
			Object.entries(
				await fetch(API_ROUTES.project_builds_current)
					.then((v) => v.json())
					.catch((e) => {
						console.error(e);
						return {};
					})
			)
		);
		projects_building = [];
		build_status.forEach((v, _k) => {
			projects_building.push(v);
		});
	}
	const t_id = setInterval(() => retrieveBuildstatus(), 1000);

	onDestroy(() => {
		clearInterval(t_id);
	});
</script>

<div class="flex items-center justify-center p-4">
	<Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<Table divClass="w-full">
			<caption class="bg-white p-5 text-left text-lg font-semibold text-gray-900 dark:bg-gray-800 dark:text-white">
				<div class="flex items-center">
					<Heading>Projects</Heading>
					<A href={routes.project_create}>
						<PlusOutline size="sm" />
					</A>
				</div>
				<Secondary class="mt-1 text-sm font-normal text-gray-500 dark:text-gray-400">All projects on this box.</Secondary>
			</caption>
			<TableHead>
				<TableHeadCell>Status</TableHeadCell>
				<TableHeadCell>Name</TableHeadCell>
				<TableHeadCell>Based on</TableHeadCell>
				<TableHeadCell>Project kind</TableHeadCell>
				<TableHeadCell>Path on disk</TableHeadCell>
			</TableHead>
			<TableBody>
				{#each $projects as project}
					<ProjectCard
						Building={!!projects_building.find((v) => v.name === project.project_name && v.branch === project.branch)}
						{project}
					/>
				{/each}
			</TableBody>
		</Table>
	</Card>
</div>
