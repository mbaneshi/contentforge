<script lang="ts">
	import type { ContentStatus, ScheduleStatus } from '$lib/types';

	interface Props {
		status: ContentStatus | ScheduleStatus;
	}

	let { status }: Props = $props();

	const colors: Record<string, string> = {
		idea: 'bg-purple-500/15 text-purple-400 ring-purple-500/30',
		drafting: 'bg-yellow-500/15 text-yellow-400 ring-yellow-500/30',
		review: 'bg-orange-500/15 text-orange-400 ring-orange-500/30',
		ready: 'bg-cyan-500/15 text-cyan-400 ring-cyan-500/30',
		scheduled: 'bg-blue-500/15 text-blue-400 ring-blue-500/30',
		published: 'bg-emerald-500/15 text-emerald-400 ring-emerald-500/30',
		archived: 'bg-zinc-500/15 text-zinc-400 ring-zinc-500/30',
		pending: 'bg-amber-500/15 text-amber-400 ring-amber-500/30',
		in_progress: 'bg-blue-500/15 text-blue-400 ring-blue-500/30',
		failed: 'bg-red-500/15 text-red-400 ring-red-500/30',
		cancelled: 'bg-zinc-500/15 text-zinc-400 ring-zinc-500/30'
	};

	let colorClass = $derived(colors[status] ?? 'bg-zinc-500/15 text-zinc-400 ring-zinc-500/30');
	let label = $derived(status.replace('_', ' ').replace(/\b\w/g, (c) => c.toUpperCase()));
</script>

<span
	class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ring-1 ring-inset {colorClass}"
>
	{label}
</span>
