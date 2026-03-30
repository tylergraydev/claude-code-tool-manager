export interface SubAgent {
	id: number;
	name: string;
	description: string;
	content: string;
	tools?: string[];
	model?: string;
	permissionMode?: string;
	skills?: string[];
	tags?: string[];
	source: string;
	sourcePath?: string;
	isFavorite: boolean;
	disallowedTools?: string[];
	maxTurns?: number;
	memory?: string;
	background?: boolean;
	effort?: string;
	isolation?: string;
	hooks?: string;
	mcpServers?: string;
	initialPrompt?: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateSubAgentRequest {
	name: string;
	description: string;
	content: string;
	tools?: string[];
	model?: string;
	permissionMode?: string;
	skills?: string[];
	tags?: string[];
	disallowedTools?: string[];
	maxTurns?: number;
	memory?: string;
	background?: boolean;
	effort?: string;
	isolation?: string;
	hooks?: string;
	mcpServers?: string;
	initialPrompt?: string;
}

export interface ProjectSubAgent {
	id: number;
	subagentId: number;
	subagent: SubAgent;
	isEnabled: boolean;
}

export interface GlobalSubAgent {
	id: number;
	subagentId: number;
	subagent: SubAgent;
	isEnabled: boolean;
}
