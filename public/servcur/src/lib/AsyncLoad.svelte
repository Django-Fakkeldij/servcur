<script lang="ts" generics="A, T extends Promise<Response>">
	import { Spinner } from 'flowbite-svelte';
	import { setContext } from 'svelte';
	import { writable } from 'svelte/store';

	export let data: T;

	const content = writable<A>(undefined);

	setContext('data', content);
	const stream = async () => {
		content.set(await data.then((val) => val.json()));
	};
</script>

{#await stream()}
	<div class="m-20 flex content-center justify-center">
		<Spinner />
	</div>
{:then}
	<slot />
{/await}
