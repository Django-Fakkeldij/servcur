<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';

	export let OnAction: () => Promise<void> = async () => {};
	export let OnActionColor: Button['$$prop_def']['color'] = 'red';

	let clicked = 0;
	$: loading = false;
	async function checkclicked(c: number) {
		loading = true;
		if (c >= 2) {
			clicked = 0;
			await OnAction();
		}
		loading = false;
	}
	$: checkclicked(clicked);
</script>

{#if loading}
	<Spinner />
{:else}
	<Button class="py-2" color={clicked === 1 ? OnActionColor : 'primary'} on:click={() => (clicked += 1)}><slot /></Button>
{/if}
