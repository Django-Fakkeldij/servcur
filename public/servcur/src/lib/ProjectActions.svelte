<script lang="ts">
	import { Button, ButtonGroup, Spinner } from 'flowbite-svelte';

	export let OnStart: () => Promise<void> = async () => {};
	export let OnRestart: () => Promise<void> = async () => {};
	export let OnStop: () => Promise<void> = async () => {};

	$: loading = false;
</script>

{#if loading}
	<Spinner />
{:else}
	<ButtonGroup>
		<Button
			class="py-2"
			color="green"
			on:click={async () => {
				loading = true;
				await OnStart();
				loading = false;
			}}>Start</Button
		>
		<Button
			class="py-2"
			color="yellow"
			on:click={async () => {
				loading = true;
				await OnRestart();
				loading = false;
			}}>Restart</Button
		>
		<Button
			class="py-2"
			color="red"
			on:click={async () => {
				loading = true;
				await OnStop();
				loading = false;
			}}>Stop</Button
		>
	</ButtonGroup>
{/if}
