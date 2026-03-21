<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { getContent, updateContent, deleteContent, publishContent } from '$lib/api';
	import type { Content, Platform } from '$lib/types';
	import { PLATFORMS, contentTypeLabel, platformLabel } from '$lib/types';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import PlatformIcon from '$lib/components/PlatformIcon.svelte';
	import MarkdownPreview from '$lib/components/MarkdownPreview.svelte';

	let content: Content | null = $state(null);
	let loading = $state(true);
	let error: string | null = $state(null);
	let editing = $state(false);
	let saving = $state(false);
	let publishing = $state(false);
	let showPlatformMenu = $state(false);
	let publishError: string | null = $state(null);
	let publishSuccess: string | null = $state(null);

	// Edit fields
	let editTitle = $state('');
	let editBody = $state('');
	let editTagsInput = $state('');
	let editProject = $state('');

	let contentId = $derived(page.params.id);

	let charCount = $derived(editing ? editBody.length : (content?.body.length ?? 0));
	let wordCount = $derived(() => {
		const text = editing ? editBody : (content?.body ?? '');
		return text.trim() === '' ? 0 : text.trim().split(/\s+/).length;
	});

	let adaptedPlatforms = $derived(
		content?.adaptations.map((a) => a.platform) ?? []
	);

	let unadaptedPlatforms = $derived(
		PLATFORMS.filter((p) => !adaptedPlatforms.includes(p.value))
	);

	$effect(() => {
		if (contentId) {
			loadContent(contentId);
		}
	});

	async function loadContent(id: string) {
		loading = true;
		error = null;
		try {
			content = await getContent(id);
			if (content) {
				editTitle = content.title;
				editBody = content.body;
				editTagsInput = content.tags.join(', ');
				editProject = content.project ?? '';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load content';
		} finally {
			loading = false;
		}
	}

	function startEditing() {
		if (!content) return;
		editTitle = content.title;
		editBody = content.body;
		editTagsInput = content.tags.join(', ');
		editProject = content.project ?? '';
		editing = true;
	}

	function cancelEditing() {
		editing = false;
	}

	async function saveEdits() {
		if (!content) return;
		saving = true;
		error = null;
		try {
			const tags = editTagsInput
				.split(',')
				.map((t) => t.trim())
				.filter((t) => t.length > 0);
			await updateContent(content.id, {
				title: editTitle.trim(),
				body: editBody.trim(),
				tags,
				project: editProject.trim() || undefined
			});
			editing = false;
			await loadContent(content.id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save';
		} finally {
			saving = false;
		}
	}

	async function handleDelete() {
		if (!content || !confirm('Are you sure you want to delete this content?')) return;
		try {
			await deleteContent(content.id);
			goto('/draft');
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to delete';
		}
	}

	async function handlePublish() {
		if (!content) return;
		publishing = true;
		publishError = null;
		publishSuccess = null;
		try {
			const result = await publishContent(content.id);
			publishSuccess = `Published successfully! ${result.results?.length ?? 0} platform(s)`;
			await loadContent(content.id);
		} catch (e) {
			publishError = e instanceof Error ? e.message : 'Failed to publish';
		} finally {
			publishing = false;
		}
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>{content?.title ?? 'Loading...'} - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-6xl">
	{#if loading}
		<div class="flex items-center justify-center py-20">
			<div class="h-8 w-8 animate-spin rounded-full border-2 border-brand-500 border-t-transparent"></div>
		</div>
	{:else if error && !content}
		<div class="rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-400">
			{error}
		</div>
		<a href="/draft" class="mt-4 inline-block text-sm text-brand-400 hover:text-brand-300">Back to content list</a>
	{:else if content}
		<!-- Header -->
		<div class="mb-6 flex items-start justify-between gap-4">
			<div class="flex items-start gap-3">
				<a
					href="/draft"
					class="mt-1 rounded-lg border border-zinc-800 bg-zinc-900 p-2 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200"
				>
					<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
						<path d="m12 19-7-7 7-7" /><path d="M19 12H5" />
					</svg>
				</a>
				<div>
					{#if editing}
						<input
							type="text"
							bind:value={editTitle}
							class="mb-1 w-full rounded-lg border border-zinc-700 bg-zinc-900 px-3 py-1.5 text-xl font-bold text-zinc-100 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
						/>
					{:else}
						<h1 class="text-2xl font-bold text-zinc-100">{content.title}</h1>
					{/if}
					<div class="mt-1.5 flex items-center gap-3">
						<StatusBadge status={content.status} />
						<span class="rounded-md bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
							{contentTypeLabel(content.content_type)}
						</span>
						{#if content.project}
							<span class="text-xs text-zinc-500">{content.project}</span>
						{/if}
						<span class="text-xs text-zinc-600">Updated {formatDate(content.updated_at)}</span>
					</div>
				</div>
			</div>
			<div class="flex items-center gap-2">
				{#if editing}
					<button
						onclick={cancelEditing}
						class="rounded-lg border border-zinc-700 px-4 py-2 text-sm font-medium text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200"
					>
						Cancel
					</button>
					<button
						onclick={saveEdits}
						disabled={saving}
						class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-500 disabled:opacity-50"
					>
						{saving ? 'Saving...' : 'Save'}
					</button>
				{:else}
					<button
						onclick={startEditing}
						class="rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm font-medium text-zinc-300 hover:bg-zinc-700"
					>
						Edit
					</button>
					<button
						onclick={handleDelete}
						class="rounded-lg border border-red-500/20 px-4 py-2 text-sm font-medium text-red-400 hover:bg-red-500/10"
					>
						Delete
					</button>
				{/if}
			</div>
		</div>

		{#if error}
			<div class="mb-4 rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
				{error}
			</div>
		{/if}

		{#if publishSuccess}
			<div class="mb-4 rounded-xl border border-emerald-500/20 bg-emerald-500/10 p-3 text-sm text-emerald-400">
				{publishSuccess}
			</div>
		{/if}

		{#if publishError}
			<div class="mb-4 rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
				{publishError}
			</div>
		{/if}

		<!-- Tags -->
		{#if content.tags.length > 0}
			<div class="mb-4 flex flex-wrap gap-2">
				{#if editing}
					<div class="w-full">
						<label for="edit-tags" class="mb-1 block text-xs text-zinc-500">Tags (comma separated)</label>
						<input
							id="edit-tags"
							type="text"
							bind:value={editTagsInput}
							class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
						/>
					</div>
				{:else}
					{#each content.tags as tag}
						<span class="rounded-full bg-brand-500/10 px-2.5 py-0.5 text-xs font-medium text-brand-400 ring-1 ring-inset ring-brand-500/20">
							#{tag}
						</span>
					{/each}
				{/if}
			</div>
		{/if}

		<div class="grid grid-cols-3 gap-6">
			<!-- Main Content -->
			<div class="col-span-2">
				{#if editing}
					<div class="grid grid-cols-2 gap-4" style="height: calc(100vh - 320px);">
						<div class="flex flex-col">
							<span class="mb-2 text-xs font-medium text-zinc-500">Markdown</span>
							<textarea
								bind:value={editBody}
								class="flex-1 resize-none rounded-xl border border-zinc-800 bg-zinc-900 p-4 font-mono text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
							></textarea>
						</div>
						<div class="flex flex-col">
							<span class="mb-2 text-xs font-medium text-zinc-500">Preview</span>
							<div class="flex-1 overflow-y-auto rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
								<MarkdownPreview source={editBody} />
							</div>
						</div>
					</div>
				{:else}
					<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-6">
						<MarkdownPreview source={content.body} />
					</div>
				{/if}

				{#if !editing}
					<div class="mt-2 text-right text-xs text-zinc-600">
						{content.body.length} chars / {content.body.trim().split(/\s+/).length} words
					</div>
				{/if}
			</div>

			<!-- Sidebar -->
			<div class="space-y-4">
				<!-- Publish Action -->
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
					<h3 class="mb-3 text-sm font-semibold text-zinc-300">Publish</h3>
					<button
						onclick={handlePublish}
						disabled={publishing}
						class="w-full rounded-lg bg-emerald-600 px-4 py-2.5 text-sm font-medium text-white transition-colors hover:bg-emerald-500 disabled:opacity-50"
					>
						{publishing ? 'Publishing...' : 'Publish to All Platforms'}
					</button>
				</div>

				<!-- Adaptations -->
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
					<h3 class="mb-3 text-sm font-semibold text-zinc-300">Platform Adaptations</h3>
					{#if content.adaptations.length === 0}
						<p class="mb-3 text-xs text-zinc-600">No platform adaptations yet.</p>
					{:else}
						<div class="mb-3 space-y-2">
							{#each content.adaptations as adaptation}
								<div class="flex items-center justify-between rounded-lg bg-zinc-800/50 p-2.5">
									<div class="flex items-center gap-2">
										<PlatformIcon platform={adaptation.platform} size="sm" />
										<span class="text-xs font-medium text-zinc-300">
											{platformLabel(adaptation.platform)}
										</span>
									</div>
									<span class="text-[10px] text-zinc-500">
										{adaptation.body.length} chars
									</span>
								</div>
							{/each}
						</div>
					{/if}

					{#if unadaptedPlatforms.length > 0}
						<div class="relative">
							<button
								onclick={() => (showPlatformMenu = !showPlatformMenu)}
								class="w-full rounded-lg border border-dashed border-zinc-700 px-3 py-2 text-xs text-zinc-500 transition-colors hover:border-zinc-600 hover:text-zinc-400"
							>
								Adapt for platform...
							</button>
							{#if showPlatformMenu}
								<div class="absolute left-0 top-full z-10 mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-800 py-1 shadow-xl">
									{#each unadaptedPlatforms as p}
										<button
											onclick={() => {
												showPlatformMenu = false;
											}}
											class="flex w-full items-center gap-2 px-3 py-2 text-left text-xs text-zinc-300 hover:bg-zinc-700"
										>
											<PlatformIcon platform={p.value} size="sm" />
											{p.label}
										</button>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</div>

				<!-- Info -->
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
					<h3 class="mb-3 text-sm font-semibold text-zinc-300">Details</h3>
					<dl class="space-y-2 text-xs">
						<div class="flex justify-between">
							<dt class="text-zinc-500">ID</dt>
							<dd class="font-mono text-zinc-400">{content.id.slice(0, 8)}...</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-zinc-500">Created</dt>
							<dd class="text-zinc-400">{formatDate(content.created_at)}</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-zinc-500">Updated</dt>
							<dd class="text-zinc-400">{formatDate(content.updated_at)}</dd>
						</div>
						<div class="flex justify-between">
							<dt class="text-zinc-500">Type</dt>
							<dd class="text-zinc-400">{contentTypeLabel(content.content_type)}</dd>
						</div>
						{#if content.project}
							<div class="flex justify-between">
								<dt class="text-zinc-500">Project</dt>
								<dd class="text-zinc-400">{content.project}</dd>
							</div>
						{/if}
						<div class="flex justify-between">
							<dt class="text-zinc-500">Media</dt>
							<dd class="text-zinc-400">{content.media.length} files</dd>
						</div>
					</dl>
				</div>
			</div>
		</div>
	{/if}
</div>
