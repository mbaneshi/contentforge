<script lang="ts">
	import { listContent, getAnalytics } from '$lib/api';
	import type { Content, AnalyticsDashboard } from '$lib/types';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import PlatformIcon from '$lib/components/PlatformIcon.svelte';
	import ContentCard from '$lib/components/ContentCard.svelte';

	let analytics: AnalyticsDashboard | null = $state(null);
	let allContent: Content[] = $state([]);
	let loading = $state(true);
	let error: string | null = $state(null);

	let recentContent = $derived(
		[...allContent]
			.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
			.slice(0, 6)
	);

	let publishedContent = $derived(
		allContent.filter((c) => c.status === 'published')
	);

	let statusCounts = $derived({
		idea: allContent.filter((c) => c.status === 'idea').length,
		drafting: allContent.filter((c) => c.status === 'drafting').length,
		review: allContent.filter((c) => c.status === 'review').length,
		ready: allContent.filter((c) => c.status === 'ready').length,
		scheduled: allContent.filter((c) => c.status === 'scheduled').length,
		published: allContent.filter((c) => c.status === 'published').length,
		archived: allContent.filter((c) => c.status === 'archived').length
	});

	$effect(() => {
		loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const [contentResult, analyticsResult] = await Promise.allSettled([
				listContent(),
				getAnalytics()
			]);
			if (contentResult.status === 'fulfilled') allContent = contentResult.value;
			if (analyticsResult.status === 'fulfilled') analytics = analyticsResult.value;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		} finally {
			loading = false;
		}
	}

	const pipelineCards = [
		{ key: 'idea' as const, label: 'Ideas', color: 'from-purple-500/20 to-purple-600/5 border-purple-500/20', iconBg: 'bg-purple-500/20 text-purple-400' },
		{ key: 'drafting' as const, label: 'Drafting', color: 'from-yellow-500/20 to-yellow-600/5 border-yellow-500/20', iconBg: 'bg-yellow-500/20 text-yellow-400' },
		{ key: 'ready' as const, label: 'Ready', color: 'from-cyan-500/20 to-cyan-600/5 border-cyan-500/20', iconBg: 'bg-cyan-500/20 text-cyan-400' },
		{ key: 'published' as const, label: 'Published', color: 'from-emerald-500/20 to-emerald-600/5 border-emerald-500/20', iconBg: 'bg-emerald-500/20 text-emerald-400' }
	];
</script>

<svelte:head>
	<title>Dashboard - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-6xl">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-zinc-100">Dashboard</h1>
			<p class="mt-1 text-sm text-zinc-500">Overview of your content pipeline</p>
		</div>
		<div class="flex gap-3">
			<a
				href="/draft/new"
				class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-brand-500"
			>
				New Draft
			</a>
			<a
				href="/schedule"
				class="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-300 transition-colors hover:bg-zinc-700 hover:text-zinc-100"
			>
				View Schedule
			</a>
		</div>
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
		<!-- Pipeline Cards -->
		<div class="mb-8 grid grid-cols-4 gap-4">
			{#each pipelineCards as card}
				<div
					class="rounded-xl border bg-gradient-to-br p-5 {card.color}"
				>
					<div class="mb-3 flex items-center justify-between">
						<span class="text-sm font-medium text-zinc-400">{card.label}</span>
						<div class="flex h-8 w-8 items-center justify-center rounded-lg {card.iconBg}">
							<span class="text-sm font-bold">{statusCounts[card.key]}</span>
						</div>
					</div>
					<p class="text-3xl font-bold text-zinc-100">{statusCounts[card.key]}</p>
				</div>
			{/each}
		</div>

		<!-- Stats Row -->
		{#if analytics}
			<div class="mb-8 grid grid-cols-3 gap-4">
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
					<p class="text-sm text-zinc-500">Total Content</p>
					<p class="mt-1 text-2xl font-bold text-zinc-100">{analytics.total_content}</p>
				</div>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
					<p class="text-sm text-zinc-500">Scheduled</p>
					<p class="mt-1 text-2xl font-bold text-zinc-100">{analytics.scheduled_count}</p>
				</div>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
					<p class="text-sm text-zinc-500">In Draft</p>
					<p class="mt-1 text-2xl font-bold text-zinc-100">{analytics.draft_count}</p>
				</div>
			</div>
		{/if}

		<!-- Recent Content -->
		<div class="mb-8">
			<div class="mb-4 flex items-center justify-between">
				<h2 class="text-lg font-semibold text-zinc-100">Recent Content</h2>
				<a href="/draft" class="text-sm text-brand-400 hover:text-brand-300">View all</a>
			</div>
			{#if recentContent.length === 0}
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-8 text-center">
					<p class="text-sm text-zinc-500">No content yet. Create your first draft to get started.</p>
					<a
						href="/draft/new"
						class="mt-3 inline-block rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-500"
					>
						Create Draft
					</a>
				</div>
			{:else}
				<div class="grid grid-cols-2 gap-3">
					{#each recentContent as item (item.id)}
						<ContentCard content={item} />
					{/each}
				</div>
			{/if}
		</div>

		<!-- Published Content -->
		{#if publishedContent.length > 0}
			<div>
				<h2 class="mb-4 text-lg font-semibold text-zinc-100">Recent Publications</h2>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50">
					<div class="divide-y divide-zinc-800">
						{#each publishedContent.slice(0, 5) as item (item.id)}
							<div class="flex items-center gap-4 p-4">
								<div class="flex-1">
									<a href="/draft/{item.id}" class="text-sm font-medium text-zinc-200 hover:text-brand-400">
										{item.title}
									</a>
									<p class="mt-0.5 text-xs text-zinc-500">
										Published {new Date(item.updated_at).toLocaleDateString()}
									</p>
								</div>
								<div class="flex items-center gap-1.5">
									{#each item.adaptations as adaptation}
										<PlatformIcon platform={adaptation.platform} size="sm" />
									{/each}
								</div>
								<StatusBadge status={item.status} />
							</div>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	{/if}
</div>
