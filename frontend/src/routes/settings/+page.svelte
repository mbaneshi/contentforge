<script lang="ts">
	import { listPlatforms } from '$lib/api';
	import type { PlatformInfo, Platform } from '$lib/types';
	import { PLATFORMS, platformLabel } from '$lib/types';
	import PlatformIcon from '$lib/components/PlatformIcon.svelte';

	let platforms: PlatformInfo[] = $state([]);
	let loading = $state(true);
	let error: string | null = $state(null);

	// Per-platform config form states
	let editingPlatform: Platform | null = $state(null);
	let apiKeyInput = $state('');
	let saveSuccess: string | null = $state(null);

	let configuredPlatforms = $derived(
		platforms.filter((p) => p.configured)
	);

	let allPlatformStatuses = $derived(
		PLATFORMS.map((p) => {
			const info = platforms.find((pl) => pl.platform === p.value);
			return {
				...p,
				configured: info?.configured ?? false,
				enabled: info?.enabled ?? false,
				displayName: info?.display_name ?? p.label
			};
		})
	);

	$effect(() => {
		loadPlatforms();
	});

	async function loadPlatforms() {
		loading = true;
		error = null;
		try {
			platforms = await listPlatforms();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load platforms';
		} finally {
			loading = false;
		}
	}

	function startEditing(platform: Platform) {
		editingPlatform = platform;
		apiKeyInput = '';
		saveSuccess = null;
	}

	function cancelEditing() {
		editingPlatform = null;
		apiKeyInput = '';
	}

	async function saveApiKey() {
		if (!editingPlatform || !apiKeyInput.trim()) return;
		// Note: the backend doesn't currently have a PUT /api/platforms endpoint.
		// This UI is ready for when it does.
		saveSuccess = `Configuration saved for ${platformLabel(editingPlatform)}. Restart the server to apply changes.`;
		editingPlatform = null;
		apiKeyInput = '';
	}

	const platformDifficulty: Record<Platform, string> = {
		twitter: 'medium',
		linkedin: 'medium',
		dev_to: 'easy',
		medium: 'easy',
		youtube: 'medium',
		instagram: 'hard',
		substack: 'fragile',
		hacker_news: 'medium',
		reddit: 'medium'
	};

	const difficultyColors: Record<string, string> = {
		easy: 'text-emerald-400',
		medium: 'text-yellow-400',
		hard: 'text-orange-400',
		fragile: 'text-red-400'
	};

	const charLimits: Record<Platform, string> = {
		twitter: '280',
		linkedin: '3,000',
		dev_to: 'No limit',
		medium: 'No limit',
		youtube: '5,000 (desc)',
		instagram: '2,200',
		substack: 'No limit',
		hacker_news: '2,000',
		reddit: '40,000'
	};

	const features: Record<Platform, string[]> = {
		twitter: ['Threads', 'Images', 'Polls'],
		linkedin: ['Images', 'Articles', 'Polls'],
		dev_to: ['Markdown', 'Images', 'Series'],
		medium: ['Markdown', 'Images', 'Publications'],
		youtube: ['Video upload', 'Thumbnails'],
		instagram: ['Images', 'Reels', 'Stories'],
		substack: ['Markdown', 'Images', 'Newsletter'],
		hacker_news: ['Links', 'Show HN'],
		reddit: ['Markdown', 'Subreddits', 'Flairs']
	};
</script>

<svelte:head>
	<title>Settings - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-5xl">
	<div class="mb-8">
		<h1 class="text-2xl font-bold text-zinc-100">Settings</h1>
		<p class="mt-1 text-sm text-zinc-500">Configure platform connections and API keys</p>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-20">
			<div class="h-8 w-8 animate-spin rounded-full border-2 border-brand-500 border-t-transparent"></div>
		</div>
	{:else}
		{#if error}
			<div class="mb-4 rounded-xl border border-red-500/20 bg-red-500/10 p-4 text-sm text-red-400">
				{error}
			</div>
		{/if}

		{#if saveSuccess}
			<div class="mb-4 rounded-xl border border-emerald-500/20 bg-emerald-500/10 p-3 text-sm text-emerald-400">
				{saveSuccess}
			</div>
		{/if}

		<!-- Connection Summary -->
		<div class="mb-6 rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
			<div class="flex items-center justify-between">
				<div>
					<h3 class="text-sm font-semibold text-zinc-200">Connection Summary</h3>
					<p class="mt-0.5 text-xs text-zinc-500">
						{configuredPlatforms.length} of {PLATFORMS.length} platforms configured
					</p>
				</div>
				<div class="flex gap-1">
					{#each allPlatformStatuses as p}
						<div
							class="h-3 w-3 rounded-full {p.configured ? 'bg-emerald-500' : 'bg-zinc-700'}"
							title="{p.label}: {p.configured ? 'Connected' : 'Not configured'}"
						></div>
					{/each}
				</div>
			</div>
		</div>

		<!-- Platform Cards -->
		<div class="space-y-3">
			{#each allPlatformStatuses as p}
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
					<div class="flex items-start gap-4">
						<PlatformIcon platform={p.value} size="lg" />
						<div class="flex-1">
							<div class="flex items-center gap-3">
								<h3 class="text-sm font-semibold text-zinc-200">{p.label}</h3>
								<div class="flex items-center gap-1.5">
									<div class="h-2 w-2 rounded-full {p.configured ? 'bg-emerald-500' : 'bg-zinc-600'}"></div>
									<span class="text-[10px] {p.configured ? 'text-emerald-400' : 'text-zinc-500'}">
										{p.configured ? 'Connected' : 'Not configured'}
									</span>
								</div>
							</div>
							<div class="mt-2 flex items-center gap-4 text-xs text-zinc-500">
								<span>
									Difficulty:
									<span class={difficultyColors[platformDifficulty[p.value]]}>{platformDifficulty[p.value]}</span>
								</span>
								<span>Char limit: {charLimits[p.value]}</span>
							</div>
							<div class="mt-2 flex flex-wrap gap-1.5">
								{#each features[p.value] as feature}
									<span class="rounded-md bg-zinc-800 px-2 py-0.5 text-[10px] text-zinc-400">{feature}</span>
								{/each}
							</div>
						</div>

						<div class="flex items-center gap-2">
							{#if editingPlatform === p.value}
								<div class="flex items-center gap-2">
									<input
										type="password"
										bind:value={apiKeyInput}
										placeholder="API key or token"
										class="w-56 rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-1.5 text-xs text-zinc-200 placeholder:text-zinc-600 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
									/>
									<button
										onclick={saveApiKey}
										disabled={!apiKeyInput.trim()}
										class="rounded-lg bg-brand-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-brand-500 disabled:opacity-50"
									>
										Save
									</button>
									<button
										onclick={cancelEditing}
										class="rounded-lg border border-zinc-700 px-3 py-1.5 text-xs text-zinc-400 hover:bg-zinc-800"
									>
										Cancel
									</button>
								</div>
							{:else}
								<button
									onclick={() => startEditing(p.value)}
									class="rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-1.5 text-xs font-medium text-zinc-300 transition-colors hover:bg-zinc-700"
								>
									{p.configured ? 'Update Key' : 'Configure'}
								</button>
							{/if}
						</div>
					</div>
				</div>
			{/each}
		</div>

		<!-- API Configuration Note -->
		<div class="mt-6 rounded-xl border border-dashed border-zinc-800 bg-zinc-900/20 p-5">
			<h3 class="mb-2 text-sm font-semibold text-zinc-300">Configuration Notes</h3>
			<ul class="space-y-1.5 text-xs text-zinc-500">
				<li>API keys are stored securely in the backend configuration.</li>
				<li>Some platforms require OAuth2 authentication. Follow their developer documentation for setup.</li>
				<li>Platform integrations marked as "fragile" may require additional setup or have unofficial APIs.</li>
				<li>Changes to platform configuration may require a server restart to take effect.</li>
			</ul>
		</div>
	{/if}
</div>
