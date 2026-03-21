<script lang="ts">
	import type { Platform } from '$lib/types';
	import { platformLabel } from '$lib/types';

	interface Props {
		platform: Platform;
		size?: 'sm' | 'md' | 'lg';
	}

	let { platform, size = 'md' }: Props = $props();

	const platformEmojis: Record<Platform, string> = {
		twitter: 'X',
		linkedin: 'in',
		dev_to: 'D',
		medium: 'M',
		youtube: 'YT',
		instagram: 'IG',
		substack: 'S',
		hacker_news: 'HN',
		reddit: 'R'
	};

	const platformColors: Record<Platform, string> = {
		twitter: 'bg-zinc-700 text-white',
		linkedin: 'bg-blue-700 text-white',
		dev_to: 'bg-zinc-900 text-white ring-1 ring-zinc-700',
		medium: 'bg-zinc-800 text-white',
		youtube: 'bg-red-600 text-white',
		instagram: 'bg-gradient-to-tr from-purple-600 via-pink-500 to-orange-400 text-white',
		substack: 'bg-orange-500 text-white',
		hacker_news: 'bg-orange-600 text-white',
		reddit: 'bg-orange-500 text-white'
	};

	const sizeClasses = {
		sm: 'h-5 w-5 text-[9px]',
		md: 'h-7 w-7 text-[10px]',
		lg: 'h-9 w-9 text-xs'
	};
</script>

<div
	class="inline-flex items-center justify-center rounded-md font-bold {sizeClasses[size]} {platformColors[platform]}"
	title={platformLabel(platform)}
>
	{platformEmojis[platform]}
</div>
