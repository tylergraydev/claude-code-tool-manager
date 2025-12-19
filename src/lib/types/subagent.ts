export interface SubAgent {
	id: number;
	name: string;
	description: string;
	content: string;
	tools?: string[];
	model?: string;
	tags?: string[];
	source: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateSubAgentRequest {
	name: string;
	description: string;
	content: string;
	tools?: string[];
	model?: string;
	tags?: string[];
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
