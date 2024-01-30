<script lang="ts">
	import type { SystemInfo } from '$lib/docker_types/__generated';
	import { dateString, fileSizeMagnitudeBytes } from '$lib/util';
	import { Card, Heading, Hr, P } from 'flowbite-svelte';
	import { InfoCircleSolid, LightbulbSolid } from 'flowbite-svelte-icons';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: system = getContext('data') as Writable<SystemInfo>;

	$: memory = fileSizeMagnitudeBytes($system.MemTotal ?? 0);
</script>

<div class="flex items-center justify-center p-4">
	<div class="flex w-full max-w-screen-xl flex-col items-center justify-center gap-4">
		<Heading>Hi there!</Heading>
		<div class="grid w-full grid-cols-2 gap-4">
			<div class="flex flex-col gap-2">
				<Card class="max-w-full">
					<div class="flex items-center gap-2">
						<LightbulbSolid />
						<Heading tag="h3">Containers:</Heading>
					</div>
					<Hr hrClass="my-4" />
					<div class="ml-4 grid grid-cols-3">
						<div>
							<Heading tag="h5">Running:</Heading>
							<P weight="bold" class="!text-green-500">{$system.ContainersRunning}</P>
						</div>
						<div>
							<Heading tag="h5">Paused:</Heading>
							<P weight="bold" class="!text-yellow-500">{$system.ContainersPaused}</P>
						</div>
						<div>
							<Heading tag="h5">Stopped:</Heading>
							<P weight="bold" class="!text-red-500">{$system.ContainersStopped}</P>
						</div>
					</div>
				</Card>
				<Card class="max-w-full">
					<div class="flex items-center gap-2">
						<LightbulbSolid />
						<Heading tag="h3">Images: {$system.Images}</Heading>
					</div>
				</Card>
				<Card class="max-w-full">
					<div class="flex items-center gap-2">
						<LightbulbSolid />
						<Heading tag="h3">Memory: {memory[0].toFixed(1)} {memory[1]}</Heading>
					</div>
				</Card>
				<Card class="max-w-full">
					<div class="flex items-center gap-2">
						<LightbulbSolid />
						<Heading tag="h3">Warnings:</Heading>
					</div>
					<Hr hrClass="my-4" />
					<div class="max-h-48 overflow-auto bg-neutral-800 p-2">
						{#if !$system.Warnings || $system.Warnings.length === 0}
							<P weight="semibold" class="!text-green-500">No warnings!</P>
						{:else}
							{#each $system.Warnings as warning, n}
								<P weight="semibold" class="!text-yellow-500">{n + 1}: "{warning}"</P>
							{/each}
						{/if}
					</div>
				</Card>
			</div>
			<Card class="max-w-full">
				<div class="flex items-center gap-2">
					<InfoCircleSolid />
					<Heading tag="h3">System</Heading>
				</div>
				<Hr hrClass="my-4" />
				<div class="flex flex-col gap-2">
					<div>
						<Heading tag="h6">Host:</Heading>
						<P weight="semibold">{$system.Name}</P>
					</div>
					<div>
						<Heading tag="h6">Architecture:</Heading>
						<P weight="semibold">{$system.Architecture}</P>
					</div>
					<div>
						<Heading tag="h6">Logical processors:</Heading>
						<P weight="semibold">{$system.NCPU}</P>
					</div>
					<div>
						<Heading tag="h6">Operating system:</Heading>
						<P weight="semibold">{$system.OSType}</P>
					</div>
					<div>
						<Heading tag="h6">System time:</Heading>
						<P weight="semibold">{dateString(new Date($system.SystemTime ?? 0))}</P>
					</div>
				</div>
			</Card>
		</div>
	</div>
</div>
