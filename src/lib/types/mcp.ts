export type McpType = 'stdio' | 'sse' | 'http';
export type McpSource = 'manual' | 'auto-detected' | 'imported';

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
