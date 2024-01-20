<script lang="ts">
	import type { ContainerSummary } from '$lib/docker_types/__generated';
	import { P } from 'flowbite-svelte';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: containers = getContext('data') as Writable<ContainerSummary[]>;
</script>

{#each $containers as container}
	<P>{container.Names?.at(0) ?? 'no-name'}</P>
{/each}
