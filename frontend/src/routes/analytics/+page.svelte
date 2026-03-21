<script lang="ts">
	import { getAnalytics, listContent } from '$lib/api';
	import type { Content, AnalyticsDashboard, Platform, ContentType } from '$lib/types';
	import { PLATFORMS, platformLabel, contentTypeLabel } from '$lib/types';
	import PlatformIcon from '$lib/components/PlatformIcon.svelte';

	let analytics: AnalyticsDashboard | null = $state(null);
	let allContent: Content[] = $state([]);
	let loading = $state(true);
	let error: string | null = $state(null);

	// Platform distribution from adaptations
	let platformCounts = $derived(() => {
		const counts: Record<string, number> = {};
		for (const c of allContent) {
			for (const a of c.adaptations) {
				counts[a.platform] = (counts[a.platform] ?? 0) + 1;
			}
		}
		return PLATFORMS.map((p) => ({
			platform: p.value,
			label: p.label,
			count: counts[p.value] ?? 0
		})).sort((a, b) => b.count - a.count);
	});

	let maxPlatformCount = $derived(
		Math.max(1, ...platformCounts().map((p) => p.count))
	);

	// Content type distribution
	let typeCounts = $derived(() => {
		const counts: Record<string, number> = {};
		for (const c of allContent) {
			counts[c.content_type] = (counts[c.content_type] ?? 0) + 1;
		}
		return Object.entries(counts)
			.map(([type, count]) => ({ type: type as ContentType, count }))
			.sort((a, b) => b.count - a.count);
	});

	let maxTypeCount = $derived(
		Math.max(1, ...typeCounts().map((t) => t.count))
	);

	// Publication timeline (by month)
	let timeline = $derived(() => {
		const months: Record<string, number> = {};
		for (const c of allContent.filter((c) => c.status === 'published')) {
			const d = new Date(c.updated_at);
			const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
			months[key] = (months[key] ?? 0) + 1;
		}
		return Object.entries(months)
			.sort(([a], [b]) => a.localeCompare(b))
			.map(([month, count]) => ({ month, count }));
	});

	let maxTimelineCount = $derived(
		Math.max(1, ...timeline().map((t) => t.count))
	);

	// Status breakdown
	let statusBreakdown = $derived(() => {
		const counts: Record<string, number> = {};
		for (const c of allContent) {
			counts[c.status] = (counts[c.status] ?? 0) + 1;
		}
		return counts;
	});

	$effect(() => {
		loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const [analyticsResult, contentResult] = await Promise.allSettled([
				getAnalytics(),
				listContent()
			]);
			if (analyticsResult.status === 'fulfilled') analytics = analyticsResult.value;
			if (contentResult.status === 'fulfilled') allContent = contentResult.value;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load analytics';
		} finally {
			loading = false;
		}
	}

	const barColors = [
		'bg-brand-500',
		'bg-emerald-500',
		'bg-purple-500',
		'bg-amber-500',
		'bg-rose-500',
		'bg-cyan-500',
		'bg-orange-500',
		'bg-pink-500',
		'bg-teal-500'
	];

	const statusColors: Record<string, string> = {
		idea: 'bg-purple-500',
		drafting: 'bg-yellow-500',
		review: 'bg-orange-500',
		ready: 'bg-cyan-500',
		scheduled: 'bg-blue-500',
		published: 'bg-emerald-500',
		archived: 'bg-zinc-500'
	};
</script>

<svelte:head>
	<title>Analytics - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-6xl">
	<div class="mb-8">
		<h1 class="text-2xl font-bold text-zinc-100">Analytics</h1>
		<p class="mt-1 text-sm text-zinc-500">Content performance and distribution</p>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-20">
			<div class="h-8 w-8 animate-spin rounded-full border-2 border-brand-500 border-t-transparent"></div>
		</div>
	{:else if error}
		<div class="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-400">
			{error}
		</div>
	{:else}
		<!-- Overview Cards -->
		{#if analytics}
			<div class="mb-8 grid grid-cols-4 gap-4">
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
					<p class="text-sm text-zinc-500">Total Content</p>
					<p class="mt-1 text-3xl font-bold text-zinc-100">{analytics.total_content}</p>
				</div>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
					<p class="text-sm text-zinc-500">Published</p>
					<p class="mt-1 text-3xl font-bold text-emerald-400">{analytics.published_count}</p>
				</div>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
					<p class="text-sm text-zinc-500">Scheduled</p>
					<p class="mt-1 text-3xl font-bold text-blue-400">{analytics.scheduled_count}</p>
				</div>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
					<p class="text-sm text-zinc-500">Drafts</p>
					<p class="mt-1 text-3xl font-bold text-yellow-400">{analytics.draft_count}</p>
				</div>
			</div>
		{/if}

		<div class="grid grid-cols-2 gap-6">
			<!-- Posts per Platform -->
			<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
				<h3 class="mb-4 text-sm font-semibold text-zinc-200">Adaptations by Platform</h3>
				{#if platformCounts().every((p) => p.count === 0)}
					<p class="py-8 text-center text-sm text-zinc-600">No adaptations yet</p>
				{:else}
					<div class="space-y-3">
						{#each platformCounts() as item, i}
							{#if item.count > 0}
								<div class="flex items-center gap-3">
									<PlatformIcon platform={item.platform as Platform} size="sm" />
									<span class="w-20 text-xs text-zinc-400">{item.label}</span>
									<div class="flex-1">
										<div class="h-5 w-full rounded-full bg-zinc-800">
											<div
												class="flex h-5 items-center rounded-full px-2 text-[10px] font-medium text-white {barColors[i % barColors.length]}"
												style="width: {Math.max(15, (item.count / maxPlatformCount) * 100)}%"
											>
												{item.count}
											</div>
										</div>
									</div>
								</div>
							{/if}
						{/each}
					</div>
				{/if}
			</div>

			<!-- Content Type Distribution -->
			<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
				<h3 class="mb-4 text-sm font-semibold text-zinc-200">Content Types</h3>
				{#if typeCounts().length === 0}
					<p class="py-8 text-center text-sm text-zinc-600">No content yet</p>
				{:else}
					<div class="space-y-3">
						{#each typeCounts() as item, i}
							<div class="flex items-center gap-3">
								<span class="w-24 text-xs text-zinc-400">{contentTypeLabel(item.type)}</span>
								<div class="flex-1">
									<div class="h-5 w-full rounded-full bg-zinc-800">
										<div
											class="flex h-5 items-center rounded-full px-2 text-[10px] font-medium text-white {barColors[(i + 3) % barColors.length]}"
											style="width: {Math.max(15, (item.count / maxTypeCount) * 100)}%"
										>
											{item.count}
										</div>
									</div>
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Publication Timeline -->
			<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
				<h3 class="mb-4 text-sm font-semibold text-zinc-200">Publication Timeline</h3>
				{#if timeline().length === 0}
					<p class="py-8 text-center text-sm text-zinc-600">No publications yet</p>
				{:else}
					<div class="flex items-end gap-2" style="height: 160px;">
						{#each timeline() as item}
							<div class="flex flex-1 flex-col items-center gap-1">
								<span class="text-[10px] font-medium text-zinc-400">{item.count}</span>
								<div class="w-full rounded-t-md bg-brand-500" style="height: {Math.max(8, (item.count / maxTimelineCount) * 140)}px"></div>
								<span class="text-[9px] text-zinc-600">{item.month.slice(5)}/{item.month.slice(2, 4)}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Status Breakdown -->
			<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
				<h3 class="mb-4 text-sm font-semibold text-zinc-200">Content Pipeline Status</h3>
				{@const breakdown = statusBreakdown()}
				{#if Object.keys(breakdown).length === 0}
					<p class="py-8 text-center text-sm text-zinc-600">No content yet</p>
				{:else}
					<div class="space-y-3">
						{#each Object.entries(breakdown) as [status, count]}
							<div class="flex items-center gap-3">
								<div class="h-3 w-3 rounded-full {statusColors[status] ?? 'bg-zinc-500'}"></div>
								<span class="w-20 text-xs capitalize text-zinc-400">{status}</span>
								<div class="flex-1">
									<div class="h-5 w-full rounded-full bg-zinc-800">
										<div
											class="flex h-5 items-center rounded-full px-2 text-[10px] font-medium text-white {statusColors[status] ?? 'bg-zinc-500'}"
											style="width: {Math.max(15, (count / Math.max(1, allContent.length)) * 100)}%"
										>
											{count}
										</div>
									</div>
								</div>
							</div>
						{/each}
					</div>

					<!-- Total bar -->
					<div class="mt-4 flex h-3 overflow-hidden rounded-full bg-zinc-800">
						{#each Object.entries(breakdown) as [status, count]}
							<div
								class="{statusColors[status] ?? 'bg-zinc-500'} h-full"
								style="width: {(count / Math.max(1, allContent.length)) * 100}%"
								title="{status}: {count}"
							></div>
						{/each}
					</div>
				{/if}
			</div>
		</div>

		<!-- Engagement Metrics Placeholder -->
		<div class="mt-6 rounded-xl border border-dashed border-zinc-800 bg-zinc-900/20 p-8 text-center">
			<p class="text-sm font-medium text-zinc-500">Engagement Metrics</p>
			<p class="mt-1 text-xs text-zinc-600">
				Likes, shares, comments, and views will appear here once platform integrations report engagement data.
			</p>
		</div>
	{/if}
</div>
