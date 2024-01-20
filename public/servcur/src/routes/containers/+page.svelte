<script lang="ts">
	import { P } from 'flowbite-svelte';
	import { getContext } from 'svelte';
	import type { Writable } from 'svelte/store';

	$: containers = getContext('data') as Writable<[]>;
	$: console.log(containers);
</script>

{#each $containers as container}
	<P>{@html JSON.stringify(container, null, 4).replaceAll('\n', '<br/>')}</P>
{/each}
