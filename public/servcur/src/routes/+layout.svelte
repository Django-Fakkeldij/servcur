<script>
	import Navigation from '$lib/Navigation.svelte';
	import '../app.pcss';
</script>

<div class="flex">
	<div>
		<Navigation />
	</div>
	<div class="h-full w-full">
		<slot />
	</div>
</div>
