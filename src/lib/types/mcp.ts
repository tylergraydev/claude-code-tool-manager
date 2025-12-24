export type McpType = 'stdio' | 'sse' | 'http';
export type McpSource = 'manual' | 'auto-detected' | 'imported' | 'system';

export interface Mcp {
	id: number;
	name: string;
	description: string | null;
	type: McpType;

	// stdio fields
	command: string | null;
	args: string[] | null;

	// sse/http fields
	url: string | null;
	headers: Record<string, string> | null;

	// Common fields
	env: Record<string, string> | null;
	icon: string | null;
	tags: string[] | null;
	source: McpSource;
	sourcePath: string | null;
	isEnabledGlobal: boolean;

	createdAt: string;
	updatedAt: string;
}

export interface CreateMcpRequest {
	name: string;
	description?: string;
	type: McpType;
	command?: string;
	args?: string[];
	url?: string;
	headers?: Record<string, string>;
	env?: Record<string, string>;
	icon?: string;
	tags?: string[];
}

export interface UpdateMcpRequest extends CreateMcpRequest {
	id: number;
}

// MCP Testing types
export interface McpTool {
	name: string;
	description: string | null;
	inputSchema: Record<string, unknown> | null;
}

export interface McpServerInfo {
	name: string;
	version: string | null;
}

export interface McpTestResult {
	success: boolean;
	serverInfo: McpServerInfo | null;
	tools: McpTool[];
	resourcesSupported: boolean;
	promptsSupported: boolean;
	error: string | null;
	responseTimeMs: number;
}

// MCP Session types for interactive execution
export interface SessionInfo {
	id: string;
	mcpId: number;
	mcpName: string;
	mcpType: string;
	serverInfo: McpServerInfo | null;
	toolCount: number;
	createdAt: string;
	lastUsedAt: string;
}

export interface StartSessionResult {
	sessionId: string;
	serverInfo: McpServerInfo | null;
	tools: McpTool[];
	resourcesSupported: boolean;
	promptsSupported: boolean;
}

// Tool execution types
export type ToolContent =
	| { type: 'text'; text: string }
	| { type: 'image'; data: string; mimeType: string }
	| { type: 'resource'; uri: string; mimeType?: string; text?: string };

export interface ToolCallResult {
	success: boolean;
	content: ToolContent[];
	isError: boolean;
	error: string | null;
	executionTimeMs: number;
}

// Execution history entry
export interface ExecutionHistoryEntry {
	id: string;
	toolName: string;
	arguments: Record<string, unknown>;
	result: ToolCallResult;
	timestamp: Date;
}
