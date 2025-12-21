export type HookEventType =
	| 'SessionStart'
	| 'UserPromptSubmit'
	| 'PreToolUse'
	| 'PostToolUse'
	| 'Notification'
	| 'Stop'
	| 'SubagentStop'
	| 'SessionEnd';

export type HookType = 'command' | 'prompt';

export interface Hook {
	id: number;
	name: string;
	description?: string;
	eventType: HookEventType;
	matcher?: string;
	hookType: HookType;
	command?: string;
	prompt?: string;
	timeout?: number;
	tags?: string[];
	source: string;
	isTemplate: boolean;
	createdAt: string;
	updatedAt: string;
}

export interface CreateHookRequest {
	name: string;
	description?: string;
	eventType: HookEventType;
	matcher?: string;
	hookType: HookType;
	command?: string;
	prompt?: string;
	timeout?: number;
	tags?: string[];
}

export interface ProjectHook {
	id: number;
	hookId: number;
	hook: Hook;
	isEnabled: boolean;
}

export interface GlobalHook {
	id: number;
	hookId: number;
	hook: Hook;
	isEnabled: boolean;
}

// Event type metadata for UI (ordered by session lifecycle)
export const HOOK_EVENT_TYPES: {
	value: HookEventType;
	label: string;
	description: string;
	matcherHint?: string;
}[] = [
	{
		value: 'SessionStart',
		label: 'Session Start',
		description: 'Runs when a new Claude Code session starts'
	},
	{
		value: 'UserPromptSubmit',
		label: 'User Prompt Submit',
		description: 'Runs when the user submits a prompt'
	},
	{
		value: 'PreToolUse',
		label: 'Pre Tool Use',
		description: 'Runs before a tool is executed',
		matcherHint: 'Tool name pattern (e.g., Bash, Write|Edit)'
	},
	{
		value: 'PostToolUse',
		label: 'Post Tool Use',
		description: 'Runs after a tool is executed',
		matcherHint: 'Tool name pattern (e.g., Bash, Write|Edit)'
	},
	{
		value: 'Notification',
		label: 'Notification',
		description: 'Runs on system notifications',
		matcherHint: 'Notification type pattern'
	},
	{
		value: 'Stop',
		label: 'Stop',
		description: 'Runs when the main agent stops'
	},
	{
		value: 'SubagentStop',
		label: 'Subagent Stop',
		description: 'Runs when a sub-agent stops'
	},
	{
		value: 'SessionEnd',
		label: 'Session End',
		description: 'Runs when the Claude Code session ends'
	}
];
