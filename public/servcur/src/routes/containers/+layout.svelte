<script lang="ts">
	import type { ContainerSummary } from '$lib/docker_types/__generated/index.js';
	import { Spinner } from 'flowbite-svelte';
	import { setContext } from 'svelte';
	import { writable } from 'svelte/store';

	export let data;

	const content = writable<ContainerSummary[]>([]);

	setContext('data', content);
	const stream = async () => {
		content.set(await data.containers_stream.then((val) => val.json()));
	};
</script>

<div>
	{#await stream()}
		<div class="m-20 flex content-center justify-center">
			<Spinner />
		</div>
	{:then}
		<slot />
	{/await}
</div>
