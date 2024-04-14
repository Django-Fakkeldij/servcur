<script lang="ts">
	import { page } from '$app/stores';
	import { Sidebar, SidebarBrand, SidebarGroup, SidebarItem, SidebarWrapper } from 'flowbite-svelte';
	import { ChartPieSolid, CodeForkSolid, DatabaseSolid, FileCodeSolid, GridSolid, ReceiptSolid } from 'flowbite-svelte-icons';
	import { routes } from './routes';

	$: activeUrl = '/' + $page.url.pathname.split('/')[1];
	let activeClass =
		'flex items-center p-2 text-base font-normal text-primary-900 bg-primary-200 dark:bg-primary-700 rounded-lg dark:text-white hover:bg-primary-100 dark:hover:bg-gray-700';
	let nonActiveClass =
		'flex items-center p-2 text-base font-normal text-green-900 rounded-lg dark:text-white hover:bg-green-100 dark:hover:bg-green-700';

	const site = {
		name: 'ServCur',
		href: routes.home,
		img: '/favicon.png',
	};
</script>

<Sidebar {activeUrl} {activeClass} {nonActiveClass}>
	<SidebarWrapper>
		<SidebarGroup>
			<SidebarBrand {site} />
			<SidebarItem label="Dashboard" href={routes.dashboard}>
				<svelte:fragment slot="icon">
					<ChartPieSolid
						class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
					/>
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Projects" href={routes.projects}>
				<svelte:fragment slot="icon">
					<FileCodeSolid
						class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
					/>
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Containers" href={routes.containers}>
				<svelte:fragment slot="icon">
					<GridSolid
						class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
					/>
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Images" href={routes.images}>
				<svelte:fragment slot="icon">
					<ReceiptSolid
						class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
					/>
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Volumes" href={routes.volumes}>
				<svelte:fragment slot="icon">
					<DatabaseSolid
						class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
					/>
				</svelte:fragment>
			</SidebarItem>
			<SidebarItem label="Networks" href={routes.networks}>
				<svelte:fragment slot="icon">
					<CodeForkSolid
						class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
					/>
				</svelte:fragment>
			</SidebarItem>
		</SidebarGroup>
	</SidebarWrapper>
</Sidebar>
