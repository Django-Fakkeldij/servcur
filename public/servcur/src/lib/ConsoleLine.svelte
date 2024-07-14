<script lang="ts">
	import type { AnsiUp } from 'ansi_up';
	import type { ConsoleMessage } from './models/console';

	export let l: ConsoleMessage;
	export let AU: AnsiUp;

	let html = AU.ansi_to_html(l.line);
</script>

<div class={l?.class} bind:innerHTML={html} contenteditable="true">
	{l.line}
</div>
