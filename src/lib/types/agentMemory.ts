export interface AgentMemoryFileInfo {
	agentName: string;
	scope: string;
	exists: boolean;
	filePath: string;
	content: string;
	lastModified: string | null;
	sizeBytes: number | null;
}

export interface AgentMemoryEntry {
	agentName: string;
	scope: string;
	filePath: string;
	sizeBytes: number;
	lastModified: string | null;
}
