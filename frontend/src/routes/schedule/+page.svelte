<script lang="ts">
	import { listSchedule, listContent, createSchedule } from '$lib/api';
	import type { ScheduleEntry, Content, Platform } from '$lib/types';
	import { PLATFORMS, platformLabel } from '$lib/types';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import PlatformIcon from '$lib/components/PlatformIcon.svelte';

	let scheduleEntries: ScheduleEntry[] = $state([]);
	let allContent: Content[] = $state([]);
	let loading = $state(true);
	let error: string | null = $state(null);
	let showForm = $state(false);

	// New schedule form
	let formContentId = $state('');
	let formPlatform: Platform = $state('twitter');
	let formDate = $state('');
	let formTime = $state('09:00');
	let formSaving = $state(false);
	let formError: string | null = $state(null);

	let sortedEntries = $derived(
		[...scheduleEntries].sort(
			(a, b) => new Date(a.scheduled_at).getTime() - new Date(b.scheduled_at).getTime()
		)
	);

	let pendingEntries = $derived(sortedEntries.filter((e) => e.status === 'pending' || e.status === 'in_progress'));
	let completedEntries = $derived(sortedEntries.filter((e) => e.status === 'published' || e.status === 'failed' || e.status === 'cancelled'));

	$effect(() => {
		loadData();
	});

	async function loadData() {
		loading = true;
		error = null;
		try {
			const [scheduleResult, contentResult] = await Promise.allSettled([
				listSchedule(),
				listContent()
			]);
			if (scheduleResult.status === 'fulfilled') scheduleEntries = scheduleResult.value;
			if (contentResult.status === 'fulfilled') allContent = contentResult.value;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load schedule';
		} finally {
			loading = false;
		}
	}

	function getContentTitle(contentId: string): string {
		const c = allContent.find((item) => item.id === contentId);
		return c?.title ?? 'Unknown content';
	}

	async function handleCreateSchedule() {
		if (!formContentId || !formDate) return;
		formSaving = true;
		formError = null;
		try {
			const scheduledAt = new Date(`${formDate}T${formTime}:00Z`).toISOString();
			await createSchedule({
				content_id: formContentId,
				platform: formPlatform,
				scheduled_at: scheduledAt
			});
			showForm = false;
			formContentId = '';
			formDate = '';
			formTime = '09:00';
			await loadData();
		} catch (e) {
			formError = e instanceof Error ? e.message : 'Failed to create schedule entry';
		} finally {
			formSaving = false;
		}
	}

	function formatScheduleDate(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleDateString('en-US', {
			weekday: 'short',
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatScheduleTime(dateStr: string): string {
		const d = new Date(dateStr);
		return d.toLocaleTimeString('en-US', {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function isOverdue(dateStr: string, status: string): boolean {
		return status === 'pending' && new Date(dateStr) < new Date();
	}

	// Group by date
	let groupedPending = $derived(() => {
		const groups: Record<string, ScheduleEntry[]> = {};
		for (const entry of pendingEntries) {
			const dateKey = formatScheduleDate(entry.scheduled_at);
			if (!groups[dateKey]) groups[dateKey] = [];
			groups[dateKey].push(entry);
		}
		return Object.entries(groups);
	});
</script>

<svelte:head>
	<title>Schedule - ContentForge</title>
</svelte:head>

<div class="mx-auto max-w-5xl">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold text-zinc-100">Schedule</h1>
			<p class="mt-1 text-sm text-zinc-500">Plan and manage publication timing</p>
		</div>
		<button
			onclick={() => (showForm = !showForm)}
			class="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-brand-500"
		>
			{showForm ? 'Cancel' : 'New Schedule Entry'}
		</button>
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

		<!-- New Schedule Form -->
		{#if showForm}
			<div class="mb-6 rounded-xl border border-zinc-800 bg-zinc-900/50 p-5">
				<h3 class="mb-4 text-sm font-semibold text-zinc-200">Schedule New Publication</h3>
				{#if formError}
					<div class="mb-3 rounded-lg border border-red-500/20 bg-red-500/10 p-2 text-xs text-red-400">
						{formError}
					</div>
				{/if}
				<div class="grid grid-cols-2 gap-4">
					<div>
						<label for="schedule-content" class="mb-1.5 block text-xs font-medium text-zinc-400">Content</label>
						<select
							id="schedule-content"
							bind:value={formContentId}
							class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
						>
							<option value="">Select content...</option>
							{#each allContent as item}
								<option value={item.id}>{item.title}</option>
							{/each}
						</select>
					</div>
					<div>
						<label for="schedule-platform" class="mb-1.5 block text-xs font-medium text-zinc-400">Platform</label>
						<select
							id="schedule-platform"
							bind:value={formPlatform}
							class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
						>
							{#each PLATFORMS as p}
								<option value={p.value}>{p.label}</option>
							{/each}
						</select>
					</div>
					<div>
						<label for="schedule-date" class="mb-1.5 block text-xs font-medium text-zinc-400">Date</label>
						<input
							id="schedule-date"
							type="date"
							bind:value={formDate}
							class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
						/>
					</div>
					<div>
						<label for="schedule-time" class="mb-1.5 block text-xs font-medium text-zinc-400">Time (UTC)</label>
						<input
							id="schedule-time"
							type="time"
							bind:value={formTime}
							class="w-full rounded-lg border border-zinc-800 bg-zinc-900 px-3 py-2 text-sm text-zinc-200 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
						/>
					</div>
				</div>
				<div class="mt-4 flex justify-end">
					<button
						onclick={handleCreateSchedule}
						disabled={!formContentId || !formDate || formSaving}
						class="rounded-lg bg-brand-600 px-5 py-2 text-sm font-medium text-white hover:bg-brand-500 disabled:cursor-not-allowed disabled:opacity-50"
					>
						{formSaving ? 'Scheduling...' : 'Schedule'}
					</button>
				</div>
			</div>
		{/if}

		<!-- Upcoming -->
		<div class="mb-8">
			<h2 class="mb-4 text-lg font-semibold text-zinc-100">
				Upcoming
				{#if pendingEntries.length > 0}
					<span class="ml-2 text-sm font-normal text-zinc-500">({pendingEntries.length})</span>
				{/if}
			</h2>
			{#if pendingEntries.length === 0}
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50 p-8 text-center">
					<p class="text-sm text-zinc-500">No upcoming scheduled publications.</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each pendingEntries as entry (entry.id)}
						{@const overdue = isOverdue(entry.scheduled_at, entry.status)}
						<div
							class="flex items-center gap-4 rounded-xl border p-4 {overdue
								? 'border-red-500/20 bg-red-500/5'
								: 'border-zinc-800 bg-zinc-900/50'}"
						>
							<div class="w-24 text-center">
								<p class="text-xs font-medium text-zinc-400">
									{formatScheduleDate(entry.scheduled_at)}
								</p>
								<p class="text-lg font-bold text-zinc-200">
									{formatScheduleTime(entry.scheduled_at)}
								</p>
								{#if overdue}
									<span class="text-[10px] font-medium text-red-400">OVERDUE</span>
								{/if}
							</div>
							<div class="h-8 w-px bg-zinc-800"></div>
							<PlatformIcon platform={entry.platform} size="md" />
							<div class="flex-1">
								<p class="text-sm font-medium text-zinc-200">
									{getContentTitle(entry.content_id)}
								</p>
								<p class="mt-0.5 text-xs text-zinc-500">
									{platformLabel(entry.platform)}
									{#if entry.retries > 0}
										<span class="ml-2 text-yellow-500">{entry.retries} retries</span>
									{/if}
								</p>
							</div>
							<StatusBadge status={entry.status} />
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Completed -->
		{#if completedEntries.length > 0}
			<div>
				<h2 class="mb-4 text-lg font-semibold text-zinc-100">
					Completed
					<span class="ml-2 text-sm font-normal text-zinc-500">({completedEntries.length})</span>
				</h2>
				<div class="rounded-xl border border-zinc-800 bg-zinc-900/50">
					<div class="divide-y divide-zinc-800">
						{#each completedEntries as entry (entry.id)}
							<div class="flex items-center gap-4 p-4">
								<PlatformIcon platform={entry.platform} size="sm" />
								<div class="flex-1">
									<p class="text-sm text-zinc-300">{getContentTitle(entry.content_id)}</p>
									<p class="mt-0.5 text-xs text-zinc-600">
										{formatScheduleDate(entry.scheduled_at)} at {formatScheduleTime(entry.scheduled_at)}
									</p>
								</div>
								{#if entry.error}
									<span class="max-w-48 truncate text-xs text-red-400" title={entry.error}>
										{entry.error}
									</span>
								{/if}
								<StatusBadge status={entry.status} />
							</div>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	{/if}
</div>
