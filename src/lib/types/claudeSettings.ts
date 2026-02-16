export type ClaudeSettingsScope = 'user' | 'project' | 'local';

export interface ClaudeSettings {
	scope: string;
	model?: string;
	availableModels: string[];
	outputStyle?: string;
	language?: string;
	alwaysThinkingEnabled?: boolean;
	attributionCommit?: string;
	attributionPr?: string;
}

export interface AllClaudeSettings {
	user: ClaudeSettings;
	project?: ClaudeSettings;
	local?: ClaudeSettings;
}

export const CLAUDE_MODELS = [
	{
		value: 'claude-sonnet-4-5-20250929',
		label: 'Claude Sonnet 4.5',
		description: 'Best balance of speed and intelligence'
	},
	{
		value: 'claude-opus-4-6',
		label: 'Claude Opus 4.6',
		description: 'Most capable model for complex tasks'
	},
	{
		value: 'claude-haiku-4-5-20251001',
		label: 'Claude Haiku 4.5',
		description: 'Fastest model for simple tasks'
	}
] as const;

export const AVAILABLE_MODEL_SHORTCUTS = [
	{ value: 'sonnet', label: 'Sonnet' },
	{ value: 'opus', label: 'Opus' },
	{ value: 'haiku', label: 'Haiku' }
] as const;

export const OUTPUT_STYLES = [
	{ value: '', label: 'Not set' },
	{ value: 'concise', label: 'Concise' },
	{ value: 'verbose', label: 'Verbose' },
	{ value: 'markdown', label: 'Markdown' }
] as const;

export const COMMON_LANGUAGES = [
	{ value: '', label: 'Not set (English default)' },
	{ value: 'english', label: 'English' },
	{ value: 'spanish', label: 'Spanish' },
	{ value: 'french', label: 'French' },
	{ value: 'german', label: 'German' },
	{ value: 'italian', label: 'Italian' },
	{ value: 'portuguese', label: 'Portuguese' },
	{ value: 'japanese', label: 'Japanese' },
	{ value: 'korean', label: 'Korean' },
	{ value: 'chinese', label: 'Chinese' },
	{ value: 'russian', label: 'Russian' },
	{ value: 'arabic', label: 'Arabic' },
	{ value: 'hindi', label: 'Hindi' },
	{ value: 'dutch', label: 'Dutch' },
	{ value: 'swedish', label: 'Swedish' },
	{ value: 'polish', label: 'Polish' },
	{ value: 'turkish', label: 'Turkish' },
	{ value: 'thai', label: 'Thai' },
	{ value: 'vietnamese', label: 'Vietnamese' }
] as const;

export const CLAUDE_SETTINGS_SCOPE_LABELS: Record<
	ClaudeSettingsScope,
	{ label: string; description: string }
> = {
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
