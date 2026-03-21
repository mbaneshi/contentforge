<script lang="ts">
	import { listContent } from '$lib/api';
	import type { Content, ContentStatus } from '$lib/types';
	import { CONTENT_STATUSES, statusLabel } from '$lib/types';
	import ContentCard from '$lib/components/ContentCard.svelte';
	import StatusBadge from '$lib/components/StatusBadge.svelte';

	let allContent: Content[] = $state([]);
	let loading = $state(true);
	let error: string | null = $state(null);
	let activeFilter: ContentStatus | 'all' = $state('all');
	let searchQuery = $state('');
	let viewMode: 'cards' | 'table' = $state('cards');

	let filteredContent = $derived(
		allContent
			.filter((c) => activeFilter === 'all' || c.status === activeFilter)
			.filter(
				(c) =>
					searchQuery === '' ||
					c.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
					c.tags.some((t) => t.toLowerCase().includes(searchQuery.toLowerCase())) ||
					(c.project?.toLowerCase().includes(searchQuery.toLowerCase()) ?? false)
			)
			.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
	);

	let filterCounts = $derived(
		Object.fromEntries([
			['all', allContent.length],
			...CONTENT_STATUSES.map((s) => [s, allContent.filter((c) => c.status === s).length])
		])
	);

	$effect(() => {
		loadContent();
	});

	async function loadContent() {
		loading = true;
		error = null;
		try {
			allContent = await listContent();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load content';
		} finally {
			loading = false;
		}
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function contentTypeLabel(ct: string): string {
		return ct.replace('_', ' ').replace(/\b\w/g, (c) => c.toUpperCase());
	}
</script>

<svelte:head>
	<title>Content - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-6xl">
	<div class="mb-6 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-zinc-100">Content</h1>
			<p class="mt-1 text-sm text-zinc-500">Manage all your drafts and published content</p>
		</div>
		<a
			href="/draft/new"
			class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-brand-500"
		>
			New Draft
		</a>
	</div>

	<!-- Search & Filters -->
	<div class="mb-6 flex items-center gap-4">
		<div class="relative flex-1">
			<svg
				class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-zinc-500"
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<circle cx="11" cy="11" r="8" />
				<path d="m21 21-4.3-4.3" />
			</svg>
			<input
				type="text"
				placeholder="Search by title, tag, or project..."
				bind:value={searchQuery}
				class="w-full rounded-lg border border-zinc-800 bg-zinc-900 py-2 pl-10 pr-4 text-sm text-zinc-200 placeholder:text-zinc-600 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
			/>
		</div>
		<div class="flex rounded-lg border border-zinc-800 bg-zinc-900 p-0.5">
			<button
				onclick={() => (viewMode = 'cards')}
				class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors {viewMode === 'cards'
					? 'bg-zinc-700 text-zinc-100'
					: 'text-zinc-500 hover:text-zinc-300'}"
			>
				Cards
			</button>
			<button
				onclick={() => (viewMode = 'table')}
				class="rounded-md px-3 py-1.5 text-xs font-medium transition-colors {viewMode === 'table'
					? 'bg-zinc-700 text-zinc-100'
					: 'text-zinc-500 hover:text-zinc-300'}"
			>
				Table
			</button>
		</div>
	</div>

	<!-- Status Tabs -->
	<div class="mb-6 flex gap-1 overflow-x-auto border-b border-zinc-800 pb-px">
		<button
			onclick={() => (activeFilter = 'all')}
			class="whitespace-nowrap border-b-2 px-3 py-2 text-sm font-medium transition-colors {activeFilter === 'all'
				? 'border-brand-500 text-brand-400'
				: 'border-transparent text-zinc-500 hover:text-zinc-300'}"
		>
			All <span class="ml-1 text-xs text-zinc-600">({filterCounts['all']})</span>
		</button>
		{#each CONTENT_STATUSES as status}
			{#if (filterCounts[status] ?? 0) > 0 || status === activeFilter}
				<button
					onclick={() => (activeFilter = status)}
					class="whitespace-nowrap border-b-2 px-3 py-2 text-sm font-medium transition-colors {activeFilter === status
						? 'border-brand-500 text-brand-400'
						: 'border-transparent text-zinc-500 hover:text-zinc-300'}"
				>
					{statusLabel(status)}
					<span class="ml-1 text-xs text-zinc-600">({filterCounts[status] ?? 0})</span>
				</button>
			{/if}
		{/each}
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-20">
			<div class="h-8 w-8 animate-spin rounded-full border-2 border-brand-500 border-t-transparent"></div>
		</div>
	{:else if error}
		<div class="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-400">
			{error}
		</div>
	{:else if filteredContent.length === 0}
		<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-12 text-center">
			<p class="text-sm text-zinc-500">
				{searchQuery || activeFilter !== 'all' ? 'No content matches your filters.' : 'No content yet.'}
			</p>
			<a
				href="/draft/new"
				class="mt-3 inline-block rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-500"
			>
				Create your first draft
			</a>
		</div>
	{:else if viewMode === 'cards'}
		<div class="grid grid-cols-2 gap-3">
			{#each filteredContent as item (item.id)}
				<ContentCard content={item} />
			{/each}
		</div>
	{:else}
		<div class="overflow-hidden rounded-xl border border-zinc-800">
			<table class="w-full">
				<thead>
					<tr class="border-b border-zinc-800 bg-zinc-900/80">
						<th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500">Title</th>
						<th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500">Type</th>
						<th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500">Status</th>
						<th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500">Tags</th>
						<th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-zinc-500">Updated</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-zinc-800/50">
					{#each filteredContent as item (item.id)}
						<tr class="group transition-colors hover:bg-zinc-900/50">
							<td class="px-4 py-3">
								<a href="/draft/{item.id}" class="text-sm font-medium text-zinc-200 group-hover:text-brand-400">
									{item.title}
								</a>
								{#if item.project}
									<p class="mt-0.5 text-xs text-zinc-600">{item.project}</p>
								{/if}
							</td>
							<td class="px-4 py-3">
								<span class="rounded-md bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
									{contentTypeLabel(item.content_type)}
								</span>
							</td>
							<td class="px-4 py-3">
								<StatusBadge status={item.status} />
							</td>
							<td class="px-4 py-3">
								<div class="flex flex-wrap gap-1">
									{#each item.tags.slice(0, 3) as tag}
										<span class="text-xs text-zinc-500">#{tag}</span>
									{/each}
								</div>
							</td>
							<td class="px-4 py-3 text-xs text-zinc-500">
								{formatDate(item.updated_at)}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
