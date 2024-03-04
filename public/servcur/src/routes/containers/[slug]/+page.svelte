<script lang="ts">
	import { API_ROUTES } from '$lib/api.js';
	import { Card, Heading } from 'flowbite-svelte';
	import { afterUpdate, onDestroy } from 'svelte';
	import { writable } from 'svelte/store';

	export let data;
	const messages = writable<MessageEvent[]>([]);

	const ws = new WebSocket(API_ROUTES.containers_logs_ws(data.slug, Math.round(Date.now() / 1000) - 200));
	const total: MessageEvent[] = [];
	ws.onmessage = (ev) => {
		total.push(ev);
		messages.set(total);
	};

	onDestroy(() => {
		ws?.close();
	});

	afterUpdate(() => {
		checkObserver();
		checkScroll();
	});

	let viewElem: null | HTMLDivElement = null;
	let observer: null | IntersectionObserver = null;
	let sticky = true;
	const checkObserver = () => {
		if (!viewElem || observer) return;

		console.log('Making observer');
		observer = new window.IntersectionObserver(([entry]) => checkIsSticky(entry), {
			root: null,
			threshold: 0.01, // set offset 0.1 means trigger if atleast 10% of element in viewport
		});
		observer.observe(viewElem);
	};

	const checkIsSticky = (entry: IntersectionObserverEntry) => {
		if (entry.isIntersecting && !!viewElem) {
			sticky = true;
		} else {
			sticky = false;
		}
	};

	const checkScroll = () => {
		if (sticky) viewElem?.scrollIntoView();
	};
</script>

<div class="flex items-center justify-center p-4">
	<Card horizontal class="w-full max-w-6xl items-center gap-2 !p-4">
		<div class="flex max-h-[80vh] w-full flex-col gap-4">
			<Heading>Logs</Heading>
			<div class="min-h-80 overflow-auto overflow-x-auto text-nowrap bg-slate-950 p-2">
				{#each $messages as m}
					<div>
						{m.data}
					</div>
				{/each}
				<div bind:this={viewElem} class="h-2" />
			</div>
		</div>
	</Card>
</div>
