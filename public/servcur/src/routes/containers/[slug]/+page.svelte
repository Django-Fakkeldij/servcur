<script lang="ts">
	import { API_ROUTES } from '$lib/api.js';
	import { writable } from 'svelte/store';

	export let data;
	const messages = writable<MessageEvent[]>([]);

	const ws = new WebSocket(API_ROUTES.containers_logs_ws(data.slug));
	const total: MessageEvent[] = [];
	ws.onmessage = (ev) => {
		total.push(ev);
		messages.set(total);
		console.log(ev.data);
	};
</script>

<div>
	{#each $messages as m}
		<div>
			{m.data}
		</div>
	{/each}
</div>
