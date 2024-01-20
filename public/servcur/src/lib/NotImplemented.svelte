<script lang="ts">
	import { P } from 'flowbite-svelte';

	import { page } from '$app/stores';
</script>

<P align={'center'}>Hi, this route ( {$page.url.pathname} ) is not implemented just yet :)</P>
