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

// ─── Cost Estimation ────────────────────────────────────────────────────────

/** Per-million-token pricing for a model family */
export interface ModelPricing {
	input: number;
	output: number;
	cacheRead: number;
	cacheWrite: number;
}

/** API pricing per million tokens (as of Feb 2025) */
const OPUS_PRICING: ModelPricing = {
	input: 15,
	output: 75,
	cacheRead: 1.5,
	cacheWrite: 18.75
};

const SONNET_PRICING: ModelPricing = {
	input: 3,
	output: 15,
	cacheRead: 0.3,
	cacheWrite: 3.75
};

const HAIKU_PRICING: ModelPricing = {
	input: 0.8,
	output: 4,
	cacheRead: 0.08,
	cacheWrite: 1
};

/** Map model IDs to their pricing. Matches by substring for flexibility. */
function getPricing(modelId: string): ModelPricing {
	const lower = modelId.toLowerCase();
	if (lower.includes('opus')) return OPUS_PRICING;
	if (lower.includes('haiku')) return HAIKU_PRICING;
	// Default to Sonnet pricing for sonnet and unknown models
	return SONNET_PRICING;
}

/** Estimate the API cost for a single model's token usage */
export function estimateModelCost(
	modelId: string,
	inputTokens: number,
	outputTokens: number,
	cacheReadTokens: number,
	cacheWriteTokens: number
): number {
	const p = getPricing(modelId);
	return (
		(inputTokens / 1_000_000) * p.input +
		(outputTokens / 1_000_000) * p.output +
		(cacheReadTokens / 1_000_000) * p.cacheRead +
		(cacheWriteTokens / 1_000_000) * p.cacheWrite
	);
}

/**
 * Estimate cost for a session that may use multiple models.
 * When per-model breakdown isn't available, uses the provided modelsUsed
 * array to pick the most expensive model's pricing as a conservative estimate.
 */
export function estimateSessionCost(
	modelsUsed: string[],
	inputTokens: number,
	outputTokens: number,
	cacheReadTokens: number,
	cacheWriteTokens: number
): number {
	// Pick the most expensive model in the session for a conservative estimate
	const model = modelsUsed.length > 0 ? modelsUsed[0] : 'claude-sonnet-4-5';
	// Sort to find most expensive (opus > sonnet > haiku)
	const sorted = [...modelsUsed].sort((a, b) => {
		const pa = getPricing(a);
		const pb = getPricing(b);
		return pb.output - pa.output;
	});
	const primaryModel = sorted[0] ?? model;
	return estimateModelCost(primaryModel, inputTokens, outputTokens, cacheReadTokens, cacheWriteTokens);
}

/** Format USD cost for display */
export function formatCost(usd: number): string {
	if (usd >= 100) return `$${usd.toFixed(0)}`;
	if (usd >= 1) return `$${usd.toFixed(2)}`;
	if (usd >= 0.01) return `$${usd.toFixed(2)}`;
	if (usd >= 0.001) return `$${usd.toFixed(3)}`;
	if (usd === 0) return '$0.00';
	return `<$0.01`;
}
