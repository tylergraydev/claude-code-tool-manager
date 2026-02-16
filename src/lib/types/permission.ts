export type PermissionCategory = 'allow' | 'deny' | 'ask';
export type PermissionScope = 'user' | 'project' | 'local';
export type PermissionDefaultMode = 'default' | 'allowEdits' | 'bypassPermissions';

export interface ScopedPermissions {
	scope: string;
	allow: string[];
	deny: string[];
	ask: string[];
	defaultMode?: string;
	additionalDirectories: string[];
}

export interface AllPermissions {
	user: ScopedPermissions;
	project?: ScopedPermissions;
	local?: ScopedPermissions;
}

export interface PermissionTemplate {
	id: number;
	name: string;
	description?: string;
	category: PermissionCategory;
	rule: string;
	toolName?: string;
	tags?: string[];
	isDefault: boolean;
	createdAt: string;
	updatedAt: string;
}

// Common tool names for the rule builder
export const PERMISSION_TOOL_NAMES = [
	{ value: 'Bash', label: 'Bash', hint: 'Shell commands — e.g. Bash(npm run *)' },
	{ value: 'Read', label: 'Read', hint: 'File reading — e.g. Read(.env*)' },
	{ value: 'Edit', label: 'Edit', hint: 'File editing — e.g. Edit(src/**)' },
	{ value: 'Write', label: 'Write', hint: 'File writing — e.g. Write(src/**)' },
	{ value: 'WebFetch', label: 'WebFetch', hint: 'URL fetching — e.g. WebFetch(https://*)' },
	{ value: 'WebSearch', label: 'WebSearch', hint: 'Web searching' },
	{ value: 'Task', label: 'Task', hint: 'Sub-agent task spawning' },
	{ value: 'mcp__', label: 'MCP Tool', hint: 'MCP tools — e.g. mcp__server__tool' }
] as const;

export const PERMISSION_DEFAULT_MODES: { value: string; label: string; description: string }[] = [
	{
		value: '',
		label: 'Not set',
		description: 'Use Claude Code default behavior'
	},
	{
		value: 'default',
		label: 'Default',
		description: 'Ask for approval on sensitive operations'
	},
	{
		value: 'allowEdits',
		label: 'Allow Edits',
		description: 'Auto-approve file edits, still ask for shell commands'
	},
	{
		value: 'bypassPermissions',
		label: 'Bypass Permissions',
		description: 'Auto-approve everything (use with caution)'
	}
];

export const PERMISSION_SCOPE_LABELS: Record<PermissionScope, { label: string; description: string }> = {
	user: {
		label: 'User',
		description: '~/.claude/settings.json — applies to all projects'
	},
	project: {
		label: 'Project',
		description: '.claude/settings.json — shared with team via git'
	},
	local: {
		label: 'Local',
		description: '.claude/settings.local.json — local overrides, not committed'
	}
};
