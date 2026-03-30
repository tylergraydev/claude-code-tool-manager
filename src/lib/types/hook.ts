export type HookEventType =
	| 'SessionStart'
	| 'InstructionsLoaded'
	| 'UserPromptSubmit'
	| 'PreToolUse'
	| 'PermissionRequest'
	| 'PostToolUse'
	| 'PostToolUseFailure'
	| 'Notification'
	| 'Stop'
	| 'StopFailure'
	| 'SubagentStart'
	| 'SubagentStop'
	| 'TaskCompleted'
	| 'TeammateIdle'
	| 'PreCompact'
	| 'PostCompact'
	| 'ConfigChange'
	| 'CwdChanged'
	| 'FileChanged'
	| 'WorktreeCreate'
	| 'WorktreeRemove'
	| 'Elicitation'
	| 'ElicitationResult'
	| 'SessionEnd';

export type HookType = 'command' | 'prompt' | 'http' | 'agent';

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
	url?: string;
	headers?: Record<string, string>;
	allowedEnvVars?: string[];
	ifCondition?: string;
	statusMessage?: string;
	once?: boolean;
	asyncMode?: boolean;
	shell?: string;
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
	url?: string;
	headers?: Record<string, string>;
	allowedEnvVars?: string[];
	ifCondition?: string;
	statusMessage?: string;
	once?: boolean;
	asyncMode?: boolean;
	shell?: string;
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
	// === Session lifecycle ===
	{
		value: 'SessionStart',
		label: 'Session Start',
		description: 'Runs when a new Claude Code session starts or resumes',
		matcherHint: 'Source type: startup, resume, clear, compact'
	},
	{
		value: 'InstructionsLoaded',
		label: 'Instructions Loaded',
		description: 'Runs when CLAUDE.md or rule files are loaded',
		matcherHint: 'Load reason: session_start, nested_traversal, path_glob_match'
	},
	// === User interaction ===
	{
		value: 'UserPromptSubmit',
		label: 'User Prompt Submit',
		description: 'Runs when the user submits a prompt, before Claude processes it'
	},
	// === Tool lifecycle ===
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
		value: 'PostToolUseFailure',
		label: 'Post Tool Use Failure',
		description: 'Runs after a tool fails. Receives error details in hook input.',
		matcherHint: 'Tool name pattern (e.g., Bash, Write|Edit, mcp__server__tool)'
	},
	// === Notifications ===
	{
		value: 'Notification',
		label: 'Notification',
		description: 'Runs when Claude Code sends notifications',
		matcherHint: 'Type: permission_prompt, idle_prompt, auth_success, elicitation_dialog'
	},
	// === Agent lifecycle ===
	{
		value: 'Stop',
		label: 'Stop',
		description: 'Runs when the main agent finishes responding. Can prevent stopping.'
	},
	{
		value: 'StopFailure',
		label: 'Stop Failure',
		description: 'Runs when the agent stops due to an error.',
		matcherHint: 'Error type: rate_limit, authentication_failed, server_error'
	},
	{
		value: 'SubagentStart',
		label: 'Subagent Start',
		description: 'Runs when a subagent is spawned.',
		matcherHint: 'Agent type name'
	},
	{
		value: 'SubagentStop',
		label: 'Subagent Stop',
		description: 'Runs when a subagent task finishes. Can prevent stopping.',
		matcherHint: 'Agent type name'
	},
	{
		value: 'TaskCompleted',
		label: 'Task Completed',
		description: 'Runs when a background task completes.'
	},
	{
		value: 'TeammateIdle',
		label: 'Teammate Idle',
		description: 'Runs before a teammate idles in agent teams.'
	},
	// === Context management ===
	{
		value: 'PreCompact',
		label: 'Pre Compact',
		description: 'Runs before a compact operation. Informational only, cannot block.',
		matcherHint: 'Trigger type: manual, auto'
	},
	{
		value: 'PostCompact',
		label: 'Post Compact',
		description: 'Runs after context compaction completes.'
	},
	// === Environment changes ===
	{
		value: 'ConfigChange',
		label: 'Config Change',
		description: 'Runs when settings files are modified.',
		matcherHint: 'Source: user_settings, project_settings, policy_settings'
	},
	{
		value: 'CwdChanged',
		label: 'Working Directory Changed',
		description: 'Runs when the working directory changes.'
	},
	{
		value: 'FileChanged',
		label: 'File Changed',
		description: 'Runs when a watched file changes.',
		matcherHint: 'Filename pattern'
	},
	// === Git worktree ===
	{
		value: 'WorktreeCreate',
		label: 'Worktree Created',
		description: 'Runs when a git worktree is created.'
	},
	{
		value: 'WorktreeRemove',
		label: 'Worktree Removed',
		description: 'Runs when a git worktree is removed.'
	},
	// === MCP elicitation ===
	{
		value: 'Elicitation',
		label: 'Elicitation',
		description: 'Runs when an MCP server requests user input.'
	},
	{
		value: 'ElicitationResult',
		label: 'Elicitation Result',
		description: 'Runs when a user responds to an MCP elicitation.'
	},
	// === Session end ===
	{
		value: 'SessionEnd',
		label: 'Session End',
		description: 'Runs when the session ends. Cannot block, for cleanup tasks only.'
	}
];
