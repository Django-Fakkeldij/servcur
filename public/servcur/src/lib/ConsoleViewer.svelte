<script lang="ts">
	import { afterUpdate } from 'svelte';
	import { writable } from 'svelte/store';

	export let port: MessagePort;

	const total = [] as string[];
	const lines = writable<string[]>([]);
	port.onmessage = (m) => {
		total.push(m.data);
		lines.set(total);
	};

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

	afterUpdate(() => {
		checkObserver();
		checkScroll();
	});
</script>

<div class="h-full overflow-auto overflow-x-auto text-nowrap rounded-md border-b-8 border-blue-500 bg-slate-950 p-2">
	{#each $lines as l}
		<div>
			{l}
		</div>
	{/each}
	<div bind:this={viewElem} class="h-2" />
</div>
