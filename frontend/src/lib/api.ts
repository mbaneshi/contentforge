import type {
	Content,
	CreateContentRequest,
	UpdateContentRequest,
	ScheduleEntry,
	CreateScheduleRequest,
	PlatformInfo,
	AnalyticsDashboard,
	Publication
} from './types';

const BASE_URL = '/api';

class ApiError extends Error {
	status: number;
	constructor(message: string, status: number) {
		super(message);
		this.name = 'ApiError';
		this.status = status;
	}
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const url = `${BASE_URL}${path}`;
	const res = await fetch(url, {
		headers: {
			'Content-Type': 'application/json',
			...options?.headers
		},
		...options
	});

	if (!res.ok) {
		const text = await res.text().catch(() => 'Unknown error');
		throw new ApiError(`${res.status}: ${text}`, res.status);
	}

	const contentType = res.headers.get('content-type');
	if (contentType && contentType.includes('application/json')) {
		return res.json();
	}
	return res.text() as unknown as T;
}

// ---- Content ----

export async function listContent(): Promise<Content[]> {
	const data = await request<{ content: Content[] }>('/content');
	return data.content ?? [];
}

export async function getContent(id: string): Promise<Content> {
	const data = await request<{ content: Content }>(`/content/${id}`);
	return data.content;
}

export async function createContent(payload: CreateContentRequest): Promise<Content> {
	return request<Content>('/content', {
		method: 'POST',
		body: JSON.stringify(payload)
	});
}

export async function updateContent(id: string, payload: UpdateContentRequest): Promise<Content> {
	return request<Content>(`/content/${id}`, {
		method: 'PUT',
		body: JSON.stringify(payload)
	});
}

export async function deleteContent(id: string): Promise<void> {
	await request(`/content/${id}`, { method: 'DELETE' });
}

export async function publishContent(id: string): Promise<{ status: string; results: Publication[] }> {
	return request(`/content/${id}/publish`, { method: 'POST' });
}

// ---- Schedule ----

export async function listSchedule(): Promise<ScheduleEntry[]> {
	const data = await request<{ schedule: ScheduleEntry[] }>('/schedule');
	return data.schedule ?? [];
}

export async function createSchedule(payload: CreateScheduleRequest): Promise<ScheduleEntry> {
	return request<ScheduleEntry>('/schedule', {
		method: 'POST',
		body: JSON.stringify(payload)
	});
}

// ---- Platforms ----

export async function listPlatforms(): Promise<PlatformInfo[]> {
	const data = await request<{ platforms: PlatformInfo[] }>('/platforms');
	return data.platforms ?? [];
}

// ---- Analytics ----

export async function getAnalytics(): Promise<AnalyticsDashboard> {
	return request<AnalyticsDashboard>('/analytics');
}
