<script lang="ts">
	import AsyncLoad from '$lib/AsyncLoad.svelte';

	export let data;
</script>

<AsyncLoad data={data.stream}>
	<slot />
</AsyncLoad>
