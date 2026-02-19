export interface StatsCacheInfo {
	filePath: string;
	exists: boolean;
	stats: StatsCache | null;
}

export interface StatsCache {
	version: number | null;
	lastComputedDate: string | null;
	dailyActivity: DailyActivity[];
	dailyModelTokens: DailyModelTokens[];
	modelUsage: Record<string, ModelUsageDetail>;
	totalSessions: number | null;
	totalMessages: number | null;
	longestSession: LongestSession | null;
	firstSessionDate: string | null;
	hourCounts: Record<string, number>;
	totalSpeculationTimeSavedMs: number | null;
}

export interface DailyActivity {
	date: string;
	messageCount: number;
	sessionCount: number;
	toolCallCount: number;
}

export interface DailyModelTokens {
	date: string;
	tokensByModel: Record<string, number>;
}

export interface ModelUsageDetail {
	inputTokens: number;
	outputTokens: number;
	cacheReadInputTokens: number;
	cacheCreationInputTokens: number;
	webSearchRequests: number;
	costUSD: number;
	contextWindow: number;
	maxOutputTokens: number;
}

export interface LongestSession {
	sessionId: string;
	duration: number;
	messageCount: number;
	timestamp: string;
}

export type DateRangeFilter = '7d' | '30d' | 'all';

/** Known model ID → color mapping */
export const MODEL_COLORS: Record<string, string> = {
	'claude-opus-4-6': '#8b5cf6',
	'claude-opus-4-5-20251101': '#7c3aed',
	'claude-sonnet-4-5-20250929': '#3b82f6',
	'claude-sonnet-4-20250514': '#60a5fa',
	'claude-haiku-4-5-20251001': '#10b981',
	'claude-haiku-3-5-20241022': '#34d399'
};

/** Fallback colors for unknown models */
export const FALLBACK_MODEL_COLORS = [
	'#f59e0b',
	'#ef4444',
	'#ec4899',
	'#14b8a6',
	'#6366f1',
	'#84cc16',
	'#f97316',
	'#06b6d4'
];

let fallbackIndex = 0;
const assignedColors: Record<string, string> = {};

export function getModelColor(modelId: string): string {
	if (MODEL_COLORS[modelId]) return MODEL_COLORS[modelId];
	if (assignedColors[modelId]) return assignedColors[modelId];
	assignedColors[modelId] = FALLBACK_MODEL_COLORS[fallbackIndex % FALLBACK_MODEL_COLORS.length];
	fallbackIndex++;
	return assignedColors[modelId];
}

export function formatModelName(modelId: string): string {
	// "claude-opus-4-6" → "Opus 4.6"
	// "claude-sonnet-4-5-20250929" → "Sonnet 4.5"
	const match = modelId.match(/claude-(\w+)-(\d+)-(\d+)(?:-\d+)?/);
	if (match) {
		const family = match[1].charAt(0).toUpperCase() + match[1].slice(1);
		return `${family} ${match[2]}.${match[3]}`;
	}
	return modelId;
}

export function formatCompactNumber(n: number): string {
	if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
	if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
	if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
	return n.toString();
}

export function formatDuration(ms: number): string {
	const totalSeconds = Math.floor(ms / 1000);
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	if (hours > 0) return `${hours}h ${minutes}m`;
	return `${minutes}m`;
}
