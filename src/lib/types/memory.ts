export type MemoryScope = 'user' | 'project' | 'local';

export interface MemoryFileInfo {
	scope: string;
	exists: boolean;
	filePath: string;
	content: string;
	lastModified?: string;
	sizeBytes?: number;
}

export interface AllMemoryFiles {
	user: MemoryFileInfo;
	project?: MemoryFileInfo;
	local?: MemoryFileInfo;
}

export const MEMORY_SCOPE_LABELS: Record<
	MemoryScope,
	{ label: string; description: string; filename: string }
> = {
	user: {
		label: 'User',
		description: '~/.claude/CLAUDE.md — global instructions for all projects',
		filename: 'CLAUDE.md'
	},
	project: {
		label: 'Project',
		description: 'CLAUDE.md — project instructions shared with team via git',
		filename: 'CLAUDE.md'
	},
	local: {
		label: 'Local',
		description: 'CLAUDE.local.md — local-only overrides, not committed',
		filename: 'CLAUDE.local.md'
	}
};
