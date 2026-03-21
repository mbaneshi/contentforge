export type ContentStatus = 'idea' | 'drafting' | 'review' | 'ready' | 'scheduled' | 'published' | 'archived';

export type ContentType = 'thread' | 'short_post' | 'article' | 'video' | 'image_post' | 'link_share';

export type Platform = 'twitter' | 'linkedin' | 'dev_to' | 'medium' | 'youtube' | 'instagram' | 'substack' | 'hacker_news' | 'reddit';

export type ScheduleStatus = 'pending' | 'in_progress' | 'published' | 'failed' | 'cancelled';

export interface Content {
	id: string;
	title: string;
	body: string;
	content_type: ContentType;
	status: ContentStatus;
	tags: string[];
	project: string | null;
	adaptations: PlatformAdaptation[];
	media: MediaAttachment[];
	created_at: string;
	updated_at: string;
}

export interface PlatformAdaptation {
	platform: Platform;
	title: string | null;
	body: string;
	thread_parts: string[] | null;
	canonical_url: string | null;
	metadata: Record<string, unknown>;
}

export interface MediaAttachment {
	id: string;
	path: string;
	mime_type: string;
	alt_text: string | null;
}

export interface Publication {
	id: string;
	content_id: string;
	platform: Platform;
	url: string;
	platform_post_id: string;
	published_at: string;
}

export interface ScheduleEntry {
	id: string;
	content_id: string;
	platform: Platform;
	scheduled_at: string;
	status: ScheduleStatus;
	error: string | null;
	retries: number;
	created_at: string;
}

export interface PlatformInfo {
	platform: Platform;
	display_name: string;
	enabled: boolean;
	configured: boolean;
}

export interface AnalyticsDashboard {
	total_content: number;
	published_count: number;
	scheduled_count: number;
	draft_count: number;
}

export interface CreateContentRequest {
	title: string;
	body: string;
	content_type: ContentType;
	tags?: string[];
	project?: string;
}

export interface UpdateContentRequest {
	title?: string;
	body?: string;
	tags?: string[];
	project?: string;
}

export interface CreateScheduleRequest {
	content_id: string;
	platform: Platform;
	scheduled_at: string;
}

export const CONTENT_STATUSES: ContentStatus[] = ['idea', 'drafting', 'review', 'ready', 'scheduled', 'published', 'archived'];

export const CONTENT_TYPES: { value: ContentType; label: string }[] = [
	{ value: 'thread', label: 'Thread' },
	{ value: 'short_post', label: 'Short Post' },
	{ value: 'article', label: 'Article' },
	{ value: 'video', label: 'Video' },
	{ value: 'image_post', label: 'Image Post' },
	{ value: 'link_share', label: 'Link Share' }
];

export const PLATFORMS: { value: Platform; label: string }[] = [
	{ value: 'twitter', label: 'Twitter/X' },
	{ value: 'linkedin', label: 'LinkedIn' },
	{ value: 'dev_to', label: 'DEV.to' },
	{ value: 'medium', label: 'Medium' },
	{ value: 'youtube', label: 'YouTube' },
	{ value: 'instagram', label: 'Instagram' },
	{ value: 'substack', label: 'Substack' },
	{ value: 'hacker_news', label: 'Hacker News' },
	{ value: 'reddit', label: 'Reddit' }
];

export function statusLabel(status: ContentStatus): string {
	return status.charAt(0).toUpperCase() + status.slice(1);
}

export function contentTypeLabel(ct: ContentType): string {
	return CONTENT_TYPES.find((t) => t.value === ct)?.label ?? ct;
}

export function platformLabel(p: Platform): string {
	return PLATFORMS.find((pl) => pl.value === p)?.label ?? p;
}
