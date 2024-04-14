<script lang="ts">
	import { Button } from 'flowbite-svelte';

	export let OnDelete: () => Promise<void> = async () => {};

	let clicked = 0;
	async function checkclicked(c: number) {
		if (c >= 2) {
			clicked = 0;
			OnDelete();
		}
	}
	$: checkclicked(clicked);
</script>

<Button class="py-2" color={clicked === 1 ? 'red' : 'primary'} on:click={() => (clicked += 1)}>X</Button>
