export interface ProjectListInfo {
	dirPath: string;
	exists: boolean;
	projects: ProjectSummary[];
}

export interface ProjectSummary {
	folderName: string;
	inferredPath: string;
	sessionCount: number;
	totalInputTokens: number;
	totalOutputTokens: number;
	totalCacheReadTokens: number;
	totalCacheCreationTokens: number;
	modelsUsed: string[];
	toolUsage: Record<string, number>;
	earliestSession: string | null;
	latestSession: string | null;
}

export interface SessionListInfo {
	projectFolder: string;
	exists: boolean;
	sessions: SessionSummary[];
}

export interface SessionSummary {
	sessionId: string;
	firstTimestamp: string | null;
	lastTimestamp: string | null;
	durationMs: number;
	userMessageCount: number;
	assistantMessageCount: number;
	totalInputTokens: number;
	totalOutputTokens: number;
	totalCacheReadTokens: number;
	totalCacheCreationTokens: number;
	modelsUsed: string[];
	gitBranch: string | null;
	cwd: string | null;
	toolCounts: Record<string, number>;
	firstUserMessage: string | null;
	version: string | null;
}

export interface SessionDetail {
	sessionId: string;
	messages: SessionMessage[];
}

export interface SessionMessage {
	uuid: string | null;
	role: string;
	timestamp: string | null;
	model: string | null;
	contentPreview: string;
	toolCalls: ToolCallInfo[];
	usage: MessageUsage | null;
}

export interface ToolCallInfo {
	toolName: string;
	toolId: string | null;
}

export interface MessageUsage {
	inputTokens: number;
	outputTokens: number;
	cacheReadInputTokens: number;
	cacheCreationInputTokens: number;
}

// ─── Helper functions ───────────────────────────────────────────────────────

export function totalTokens(s: SessionSummary): number {
	return s.totalInputTokens + s.totalOutputTokens;
}

export function projectTotalTokens(p: ProjectSummary): number {
	return p.totalInputTokens + p.totalOutputTokens;
}

export type SessionSortField = 'date' | 'tokens' | 'duration' | 'messages' | 'cost';
