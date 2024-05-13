<script lang="ts">
	import ConsoleViewer from '$lib/ConsoleViewer.svelte';
	import { API_ROUTES } from '$lib/api.js';
	import type { ConsoleMessage } from '$lib/models/console.js';
	import { dateString } from '$lib/util.js';
	import { Card, Heading } from 'flowbite-svelte';
	import { onDestroy } from 'svelte';

	export let data;

	const ws = new WebSocket(API_ROUTES.containers_logs_ws(data.slug, Math.round(Date.now() / 1000) - 200));
	const channel = new MessageChannel();
	channel.port1.start();
	channel.port1.postMessage({ line: `Connected to container logs`, class: 'text-gray-600 opacity-50' } satisfies ConsoleMessage);
	ws.onmessage = (ev) => {
		const t = dateString(new Date());
		channel.port1.postMessage({ line: t + ': ' + ev.data } satisfies ConsoleMessage);
	};

	onDestroy(() => {
		ws?.close();
	});
</script>

<div class="flex items-center justify-center p-4">
	<Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<div class="flex max-h-[80vh] w-full flex-col gap-4">
			<Heading>Logs</Heading>
			<ConsoleViewer port={channel.port2} />
		</div>
	</Card>
</div>
