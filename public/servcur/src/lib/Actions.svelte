<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';

	export let OnDelete: () => Promise<void> = async () => {};

	let clicked = 0;
	$: loading = false;
	async function checkclicked(c: number) {
		loading = true;
		if (c >= 2) {
			clicked = 0;
			await OnDelete();
		}
		loading = false;
	}
	$: checkclicked(clicked);
</script>

{#if loading}
	<Spinner />
{:else}
	<Button class="py-2" color={clicked === 1 ? 'red' : 'primary'} on:click={() => (clicked += 1)}>X</Button>
{/if}
