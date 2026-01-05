export type HookEventType =
	| 'SessionStart'
	| 'UserPromptSubmit'
	| 'PreToolUse'
	| 'PermissionRequest'
	| 'PostToolUse'
	| 'Notification'
	| 'Stop'
	| 'SubagentStop'
	| 'PreCompact'
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
		description: 'Runs when a new Claude Code session starts or resumes',
		matcherHint: 'Source type: startup, resume, clear, compact'
	},
	{
		value: 'UserPromptSubmit',
		label: 'User Prompt Submit',
		description: 'Runs when the user submits a prompt, before Claude processes it'
	},
	{
		value: 'PreToolUse',
		label: 'Pre Tool Use',
		description: 'Runs before a tool is executed. Can block or modify tool input.',
		matcherHint: 'Tool name pattern (e.g., Bash, Write|Edit, mcp__server__tool)'
	},
	{
		value: 'PermissionRequest',
		label: 'Permission Request',
		description: 'Runs when a permission dialog is shown. Can allow/deny on behalf of user.',
		matcherHint: 'Tool name pattern (e.g., Bash, Write|Edit)'
	},
	{
		value: 'PostToolUse',
		label: 'Post Tool Use',
		description: 'Runs after a tool completes successfully. Can provide feedback to Claude.',
		matcherHint: 'Tool name pattern (e.g., Bash, Write|Edit, mcp__server__tool)'
	},
	{
		value: 'Notification',
		label: 'Notification',
		description: 'Runs when Claude Code sends notifications',
		matcherHint: 'Type: permission_prompt, idle_prompt, auth_success, elicitation_dialog'
	},
	{
		value: 'Stop',
		label: 'Stop',
		description: 'Runs when the main agent finishes responding. Can prevent stopping.',
	},
	{
		value: 'SubagentStop',
		label: 'Subagent Stop',
		description: 'Runs when a subagent task finishes. Can prevent stopping.',
	},
	{
		value: 'PreCompact',
		label: 'Pre Compact',
		description: 'Runs before a compact operation. Informational only, cannot block.',
		matcherHint: 'Trigger type: manual, auto'
	},
	{
		value: 'SessionEnd',
		label: 'Session End',
		description: 'Runs when the session ends. Cannot block, for cleanup tasks only.'
	}
];
