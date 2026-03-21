<script lang="ts">
	import { listContent, listPlatforms, publishContent } from '$lib/api';
	import type { Content, PlatformInfo } from '$lib/types';
	import { platformLabel } from '$lib/types';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import PlatformIcon from '$lib/components/PlatformIcon.svelte';

	let allContent: Content[] = $state([]);
	let platforms: PlatformInfo[] = $state([]);
	let loading = $state(true);
	let error: string | null = $state(null);
	let publishingId: string | null = $state(null);
	let publishResult: { id: string; message: string; success: boolean } | null = $state(null);

	let readyContent = $derived(
		allContent.filter((c) => c.status === 'ready' || c.adaptations.length > 0)
			.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
	);

	let publishedContent = $derived(
		allContent.filter((c) => c.status === 'published')
			.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
	);

	$effect(() => {
		loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const [contentResult, platformsResult] = await Promise.allSettled([
				listContent(),
				listPlatforms()
			]);
			if (contentResult.status === 'fulfilled') allContent = contentResult.value;
			if (platformsResult.status === 'fulfilled') platforms = platformsResult.value;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		} finally {
			loading = false;
		}
	}

	async function handlePublish(id: string) {
		publishingId = id;
		publishResult = null;
		try {
			const result = await publishContent(id);
			publishResult = {
				id,
				message: `Published to ${result.results?.length ?? 0} platform(s)`,
				success: true
			};
			await loadData();
		} catch (e) {
			publishResult = {
				id,
				message: e instanceof Error ? e.message : 'Failed to publish',
				success: false
			};
		} finally {
			publishingId = null;
		}
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Publish - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-6xl">
	<div class="mb-8">
		<h1 class="text-2xl font-bold text-zinc-100">Publishing Dashboard</h1>
		<p class="mt-1 text-sm text-zinc-500">Manage and publish adapted content</p>
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
		<!-- Platform Status -->
		<div class="mb-8">
			<h2 class="mb-4 text-lg font-semibold text-zinc-100">Platform Status</h2>
			<div class="grid grid-cols-3 gap-3">
				{#each [
					{ value: 'twitter' as const, label: 'Twitter/X' },
					{ value: 'linkedin' as const, label: 'LinkedIn' },
					{ value: 'dev_to' as const, label: 'DEV.to' },
					{ value: 'medium' as const, label: 'Medium' },
					{ value: 'youtube' as const, label: 'YouTube' },
					{ value: 'instagram' as const, label: 'Instagram' },
					{ value: 'substack' as const, label: 'Substack' },
					{ value: 'hacker_news' as const, label: 'Hacker News' },
					{ value: 'reddit' as const, label: 'Reddit' }
				] as p}
					{@const configured = platforms.some((pl) => pl.platform === p.value && pl.configured)}
					<div class="flex items-center gap-3 rounded-xl border border-zinc-800 bg-zinc-900/50 p-3">
						<PlatformIcon platform={p.value} size="md" />
						<div class="flex-1">
							<p class="text-sm font-medium text-zinc-200">{p.label}</p>
						</div>
						<div class="flex items-center gap-1.5">
							<div
								class="h-2 w-2 rounded-full {configured ? 'bg-emerald-500' : 'bg-zinc-600'}"
							></div>
							<span class="text-[10px] text-zinc-500">
								{configured ? 'Connected' : 'Not configured'}
							</span>
						</div>
					</div>
				{/each}
			</div>
		</div>

		{#if publishResult}
			<div
				class="mb-4 rounded-xl border p-3 text-sm {publishResult.success
					? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-400'
					: 'border-red-500/20 bg-red-500/10 text-red-400'}"
			>
				{publishResult.message}
			</div>
		{/if}

		<!-- Ready to Publish -->
		<div class="mb-8">
			<h2 class="mb-4 text-lg font-semibold text-zinc-100">Ready to Publish</h2>
			{#if readyContent.length === 0}
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-8 text-center">
					<p class="text-sm text-zinc-500">No content ready to publish. Create and adapt content first.</p>
					<a
						href="/draft/new"
						class="mt-3 inline-block rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-500"
					>
						Create Draft
					</a>
				</div>
			{:else}
				<div class="space-y-3">
					{#each readyContent as item (item.id)}
						<div class="flex items-center gap-4 rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
							<div class="flex-1">
								<a href="/draft/{item.id}" class="text-sm font-semibold text-zinc-200 hover:text-brand-400">
									{item.title}
								</a>
								<div class="mt-1.5 flex items-center gap-2">
									<StatusBadge status={item.status} />
									<span class="text-xs text-zinc-600">{formatDate(item.updated_at)}</span>
								</div>
							</div>
							<div class="flex items-center gap-2">
								{#each item.adaptations as adaptation}
									<PlatformIcon platform={adaptation.platform} size="sm" />
								{/each}
							</div>
							<button
								onclick={() => handlePublish(item.id)}
								disabled={publishingId === item.id}
								class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-500 disabled:opacity-50"
							>
								{publishingId === item.id ? 'Publishing...' : 'Publish'}
							</button>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Publication History -->
		<div>
			<h2 class="mb-4 text-lg font-semibold text-zinc-100">Publication History</h2>
			{#if publishedContent.length === 0}
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-8 text-center">
					<p class="text-sm text-zinc-500">No publications yet.</p>
				</div>
			{:else}
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50">
					<div class="divide-y divide-zinc-800">
						{#each publishedContent as item (item.id)}
							<div class="flex items-center gap-4 p-4">
								<div class="flex-1">
									<a href="/draft/{item.id}" class="text-sm font-medium text-zinc-200 hover:text-brand-400">
										{item.title}
									</a>
									<p class="mt-0.5 text-xs text-zinc-600">
										Published {formatDate(item.updated_at)}
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
			{/if}
		</div>
	{/if}
</div>
