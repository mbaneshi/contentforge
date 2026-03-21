<script lang="ts">
	import type { Content } from '$lib/types';
	import { contentTypeLabel } from '$lib/types';
	import StatusBadge from './StatusBadge.svelte';
	import PlatformIcon from './PlatformIcon.svelte';

	interface Props {
		content: Content;
	}

	let { content }: Props = $props();

	let timeAgo = $derived(formatTimeAgo(content.updated_at));

	function formatTimeAgo(dateStr: string): string {
		const date = new Date(dateStr);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);
		if (diffMins < 1) return 'just now';
		if (diffMins < 60) return `${diffMins}m ago`;
		const diffHours = Math.floor(diffMins / 60);
		if (diffHours < 24) return `${diffHours}h ago`;
		const diffDays = Math.floor(diffHours / 24);
		if (diffDays < 30) return `${diffDays}d ago`;
		return date.toLocaleDateString();
	}
</script>

<a
	href="/draft/{content.id}"
	class="group block rounded-xl border border-zinc-800 bg-zinc-900/50 p-4 transition-all hover:border-zinc-700 hover:bg-zinc-900"
>
	<div class="mb-3 flex items-start justify-between gap-3">
		<h3
			class="text-sm font-semibold text-zinc-100 group-hover:text-brand-400 transition-colors line-clamp-1"
		>
			{content.title}
		</h3>
		<StatusBadge status={content.status} />
	</div>

	<p class="mb-3 text-xs text-zinc-500 line-clamp-2">
		{content.body.slice(0, 140)}{content.body.length > 140 ? '...' : ''}
	</p>

	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<span class="rounded-md bg-zinc-800 px-2 py-0.5 text-[10px] font-medium text-zinc-400">
				{contentTypeLabel(content.content_type)}
			</span>
			{#each content.tags.slice(0, 2) as tag}
				<span class="text-[10px] text-zinc-500">#{tag}</span>
			{/each}
			{#if content.tags.length > 2}
				<span class="text-[10px] text-zinc-600">+{content.tags.length - 2}</span>
			{/if}
		</div>
		<div class="flex items-center gap-1.5">
			{#each content.adaptations as adaptation}
				<PlatformIcon platform={adaptation.platform} size="sm" />
			{/each}
			<span class="ml-1 text-[10px] text-zinc-600">{timeAgo}</span>
		</div>
	</div>
</a>
