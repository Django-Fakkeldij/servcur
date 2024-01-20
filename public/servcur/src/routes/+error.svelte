<script>
	import { page } from '$app/stores';
	import { Heading, P } from 'flowbite-svelte';
</script>

<div class="text-center">
	<Heading tag="h1" class="mb-4" customSize="text-4xl font-extrabold  md:text-5xl lg:text-6xl">Whoops...</Heading>
	<P class="mb-6 text-center text-lg dark:text-gray-400 sm:px-16 lg:text-xl xl:px-48">{$page.status}: {$page.error?.message}</P>
</div>
