<script lang="ts">
	import { marked } from 'marked';

	interface Props {
		source: string;
	}

	let { source }: Props = $props();

	let html = $derived(renderMarkdown(source));

	function renderMarkdown(md: string): string {
		try {
			return marked.parse(md, { async: false }) as string;
		} catch {
			return '<p class="text-red-400">Error rendering markdown</p>';
		}
	}
</script>

<div class="prose-content">
	{@html html}
</div>
