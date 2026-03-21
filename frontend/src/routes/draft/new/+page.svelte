<script lang="ts">
	import { goto } from '$app/navigation';
	import { createContent } from '$lib/api';
	import type { ContentType } from '$lib/types';
	import { CONTENT_TYPES } from '$lib/types';
	import MarkdownPreview from '$lib/components/MarkdownPreview.svelte';

	let title = $state('');
	let body = $state('');
	let contentType: ContentType = $state('article');
	let tagsInput = $state('');
	let project = $state('');
	let saving = $state(false);
	let error: string | null = $state(null);

	let charCount = $derived(body.length);
	let wordCount = $derived(body.trim() === '' ? 0 : body.trim().split(/\s+/).length);
	let tags = $derived(
		tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0)
	);

	let isValid = $derived(title.trim().length > 0 && body.trim().length > 0);

	async function handleSave() {
		if (!isValid) return;
		saving = true;
		error = null;
		try {
			const result = await createContent({
				title: title.trim(),
				body: body.trim(),
				content_type: contentType,
				tags: tags.length > 0 ? tags : undefined,
				project: project.trim() || undefined
			});
			if (result?.id) {
				goto(`/draft/${result.id}`);
			} else {
				goto('/draft');
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to save draft';
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>New Draft - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-7xl">
	<div class="mb-6 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<a
				href="/draft"
				class="rounded-lg border border-zinc-800 bg-zinc-900 p-2 text-zinc-400 transition-colors hover:bg-zinc-800 hover:text-zinc-200"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
					<path d="m12 19-7-7 7-7" /><path d="M19 12H5" />
				</svg>
			</a>
			<div>
				<h1 class="text-2xl font-bold text-zinc-100">New Draft</h1>
				<p class="mt-0.5 text-sm text-zinc-500">Create new content</p>
			</div>
		</div>
		<div class="flex items-center gap-3">
			<span class="text-xs text-zinc-600">
				{charCount} chars / {wordCount} words
			</span>
			<button
				onclick={handleSave}
				disabled={!isValid || saving}
				class="rounded-lg bg-brand-600 px-5 py-2 text-sm font-medium text-white transition-colors hover:bg-brand-500 disabled:cursor-not-allowed disabled:opacity-50"
			>
				{saving ? 'Saving...' : 'Save Draft'}
			</button>
		</div>
	</div>

	{#if error}
		<div class="mb-4 rounded-xl border border-red-500/20 bg-red-500/10 p-3 text-sm text-red-400">
			{error}
		</div>
	{/if}

	<!-- Metadata Row -->
	<div class="mb-4 grid grid-cols-3 gap-4">
		<div>
			<label for="content-type" class="mb-1.5 block text-xs font-medium text-zinc-400">Content Type</label>
			<select
				id="content-type"
				bind:value={contentType}
				class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
			>
				{#each CONTENT_TYPES as ct}
					<option value={ct.value}>{ct.label}</option>
				{/each}
			</select>
		</div>
		<div>
			<label for="tags" class="mb-1.5 block text-xs font-medium text-zinc-400">Tags (comma separated)</label>
			<input
				id="tags"
				type="text"
				bind:value={tagsInput}
				placeholder="rust, svelte, webdev"
				class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 placeholder:text-zinc-600 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
			/>
		</div>
		<div>
			<label for="project" class="mb-1.5 block text-xs font-medium text-zinc-400">Project</label>
			<input
				id="project"
				type="text"
				bind:value={project}
				placeholder="e.g., contentforge"
				class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 placeholder:text-zinc-600 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
			/>
		</div>
	</div>

	<!-- Title -->
	<div class="mb-4">
		<input
			type="text"
			bind:value={title}
			placeholder="Title"
			class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-4 py-3 text-lg font-semibold text-zinc-100 placeholder:text-zinc-600 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
		/>
	</div>

	<!-- Tags Preview -->
	{#if tags.length > 0}
		<div class="mb-4 flex flex-wrap gap-2">
			{#each tags as tag}
				<span class="rounded-full bg-brand-500/10 px-2.5 py-0.5 text-xs font-medium text-brand-400 ring-1 ring-inset ring-brand-500/20">
					#{tag}
				</span>
			{/each}
		</div>
	{/if}

	<!-- Editor + Preview -->
	<div class="grid grid-cols-2 gap-4" style="height: calc(100vh - 340px);">
		<div class="flex flex-col">
			<div class="mb-2 flex items-center justify-between">
				<span class="text-xs font-medium text-zinc-500">Markdown</span>
			</div>
			<textarea
				bind:value={body}
				placeholder="Write your content in Markdown..."
				class="flex-1 resize-none rounded-xl border border-zinc-800 bg-zinc-900 p-4 font-mono text-sm text-zinc-200 placeholder:text-zinc-600 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
				spellcheck="true"
			></textarea>
		</div>
		<div class="flex flex-col">
			<div class="mb-2">
				<span class="text-xs font-medium text-zinc-500">Preview</span>
			</div>
			<div class="flex-1 overflow-y-auto rounded-xl border border-zinc-800 bg-zinc-900/50 p-4">
				{#if body.trim()}
					<MarkdownPreview source={body} />
				{:else}
					<p class="text-sm text-zinc-600 italic">Preview will appear here as you type...</p>
				{/if}
			</div>
		</div>
	</div>
</div>
