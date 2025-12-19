import type { Mcp } from './mcp';

export interface Project {
	id: number;
	name: string;
	path: string;
	hasMcpFile: boolean;
	hasSettingsFile: boolean;
	lastScannedAt: string | null;
	createdAt: string;
	updatedAt: string;
	assignedMcps: ProjectMcp[];
}

export interface ProjectMcp {
	id: number;
	mcpId: number;
	mcp: Mcp;
	isEnabled: boolean;
	envOverrides: Record<string, string> | null;
	displayOrder: number;
}

export interface GlobalMcp {
	id: number;
	mcpId: number;
	mcp: Mcp;
	isEnabled: boolean;
	envOverrides: Record<string, string> | null;
}

export interface CreateProjectRequest {
	name: string;
	path: string;
}
