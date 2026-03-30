import { describe, it, expect } from 'vitest';
import { HOOK_EVENT_TYPES } from '$lib/types/hook';
import type { HookEventType, HookType } from '$lib/types/hook';

describe('Hook Types', () => {
	describe('HOOK_EVENT_TYPES', () => {
		it('should contain all expected event types', () => {
			const eventValues = HOOK_EVENT_TYPES.map((e) => e.value);
			// Original events
			expect(eventValues).toContain('SessionStart');
			expect(eventValues).toContain('UserPromptSubmit');
			expect(eventValues).toContain('PreToolUse');
			expect(eventValues).toContain('PermissionRequest');
			expect(eventValues).toContain('PostToolUse');
			expect(eventValues).toContain('Notification');
			expect(eventValues).toContain('Stop');
			expect(eventValues).toContain('SubagentStop');
			expect(eventValues).toContain('PreCompact');
			expect(eventValues).toContain('SessionEnd');
			// New events
			expect(eventValues).toContain('InstructionsLoaded');
			expect(eventValues).toContain('PostToolUseFailure');
			expect(eventValues).toContain('StopFailure');
			expect(eventValues).toContain('SubagentStart');
			expect(eventValues).toContain('TaskCompleted');
			expect(eventValues).toContain('TeammateIdle');
			expect(eventValues).toContain('PostCompact');
			expect(eventValues).toContain('ConfigChange');
			expect(eventValues).toContain('CwdChanged');
			expect(eventValues).toContain('FileChanged');
			expect(eventValues).toContain('WorktreeCreate');
			expect(eventValues).toContain('WorktreeRemove');
			expect(eventValues).toContain('Elicitation');
			expect(eventValues).toContain('ElicitationResult');
		});

		it('should have labels and descriptions for all event types', () => {
			for (const eventType of HOOK_EVENT_TYPES) {
				expect(eventType.label).toBeDefined();
				expect(eventType.label.length).toBeGreaterThan(0);
				expect(eventType.description).toBeDefined();
				expect(eventType.description.length).toBeGreaterThan(0);
			}
		});

		it('should be in session lifecycle order', () => {
			const eventValues = HOOK_EVENT_TYPES.map((e) => e.value);
			// First should be SessionStart, last should be SessionEnd
			expect(eventValues[0]).toBe('SessionStart');
			expect(eventValues[eventValues.length - 1]).toBe('SessionEnd');
		});

		it('should have matcher hints for tool-use events', () => {
			const preToolUse = HOOK_EVENT_TYPES.find((e) => e.value === 'PreToolUse');
			const postToolUse = HOOK_EVENT_TYPES.find((e) => e.value === 'PostToolUse');
			const postToolUseFailure = HOOK_EVENT_TYPES.find((e) => e.value === 'PostToolUseFailure');

			expect(preToolUse?.matcherHint).toBeDefined();
			expect(postToolUse?.matcherHint).toBeDefined();
			expect(postToolUseFailure?.matcherHint).toBeDefined();
		});

		it('should have matcher hint for Notification event', () => {
			const notification = HOOK_EVENT_TYPES.find((e) => e.value === 'Notification');
			expect(notification?.matcherHint).toBeDefined();
		});

		it('should have matcher hint for PermissionRequest event', () => {
			const permissionRequest = HOOK_EVENT_TYPES.find((e) => e.value === 'PermissionRequest');
			expect(permissionRequest?.matcherHint).toBeDefined();
		});

		it('should have matcher hint for PreCompact event', () => {
			const preCompact = HOOK_EVENT_TYPES.find((e) => e.value === 'PreCompact');
			expect(preCompact?.matcherHint).toBeDefined();
		});

		it('should have matcher hint for SessionStart event', () => {
			const sessionStart = HOOK_EVENT_TYPES.find((e) => e.value === 'SessionStart');
			expect(sessionStart?.matcherHint).toBeDefined();
		});

		it('should have 24 event types total', () => {
			expect(HOOK_EVENT_TYPES.length).toBe(24);
		});

		it('should have matcher hints for new events that support matchers', () => {
			const instructionsLoaded = HOOK_EVENT_TYPES.find((e) => e.value === 'InstructionsLoaded');
			expect(instructionsLoaded?.matcherHint).toBeDefined();

			const stopFailure = HOOK_EVENT_TYPES.find((e) => e.value === 'StopFailure');
			expect(stopFailure?.matcherHint).toBeDefined();

			const subagentStart = HOOK_EVENT_TYPES.find((e) => e.value === 'SubagentStart');
			expect(subagentStart?.matcherHint).toBeDefined();

			const subagentStop = HOOK_EVENT_TYPES.find((e) => e.value === 'SubagentStop');
			expect(subagentStop?.matcherHint).toBeDefined();

			const configChange = HOOK_EVENT_TYPES.find((e) => e.value === 'ConfigChange');
			expect(configChange?.matcherHint).toBeDefined();

			const fileChanged = HOOK_EVENT_TYPES.find((e) => e.value === 'FileChanged');
			expect(fileChanged?.matcherHint).toBeDefined();
		});

		it('should not have matcher hints for events without matchers', () => {
			const cwdChanged = HOOK_EVENT_TYPES.find((e) => e.value === 'CwdChanged');
			expect(cwdChanged?.matcherHint).toBeUndefined();

			const postCompact = HOOK_EVENT_TYPES.find((e) => e.value === 'PostCompact');
			expect(postCompact?.matcherHint).toBeUndefined();

			const taskCompleted = HOOK_EVENT_TYPES.find((e) => e.value === 'TaskCompleted');
			expect(taskCompleted?.matcherHint).toBeUndefined();

			const teammateIdle = HOOK_EVENT_TYPES.find((e) => e.value === 'TeammateIdle');
			expect(teammateIdle?.matcherHint).toBeUndefined();

			const worktreeCreate = HOOK_EVENT_TYPES.find((e) => e.value === 'WorktreeCreate');
			expect(worktreeCreate?.matcherHint).toBeUndefined();

			const worktreeRemove = HOOK_EVENT_TYPES.find((e) => e.value === 'WorktreeRemove');
			expect(worktreeRemove?.matcherHint).toBeUndefined();

			const elicitation = HOOK_EVENT_TYPES.find((e) => e.value === 'Elicitation');
			expect(elicitation?.matcherHint).toBeUndefined();

			const elicitationResult = HOOK_EVENT_TYPES.find((e) => e.value === 'ElicitationResult');
			expect(elicitationResult?.matcherHint).toBeUndefined();
		});

		it('should group related events together in order', () => {
			const eventValues = HOOK_EVENT_TYPES.map((e) => e.value);

			// Tool events should be consecutive
			const preToolIdx = eventValues.indexOf('PreToolUse');
			const permIdx = eventValues.indexOf('PermissionRequest');
			const postToolIdx = eventValues.indexOf('PostToolUse');
			const postToolFailIdx = eventValues.indexOf('PostToolUseFailure');
			expect(permIdx).toBe(preToolIdx + 1);
			expect(postToolIdx).toBe(permIdx + 1);
			expect(postToolFailIdx).toBe(postToolIdx + 1);

			// Compact events should be consecutive
			const preCompactIdx = eventValues.indexOf('PreCompact');
			const postCompactIdx = eventValues.indexOf('PostCompact');
			expect(postCompactIdx).toBe(preCompactIdx + 1);

			// Worktree events should be consecutive
			const wtCreateIdx = eventValues.indexOf('WorktreeCreate');
			const wtRemoveIdx = eventValues.indexOf('WorktreeRemove');
			expect(wtRemoveIdx).toBe(wtCreateIdx + 1);

			// Elicitation events should be consecutive
			const elicitIdx = eventValues.indexOf('Elicitation');
			const elicitResultIdx = eventValues.indexOf('ElicitationResult');
			expect(elicitResultIdx).toBe(elicitIdx + 1);
		});
	});

	describe('Type definitions', () => {
		it('should define valid HookEventType values', () => {
			const validEventTypes: HookEventType[] = [
				'SessionStart',
				'InstructionsLoaded',
				'UserPromptSubmit',
				'PreToolUse',
				'PermissionRequest',
				'PostToolUse',
				'PostToolUseFailure',
				'Notification',
				'Stop',
				'StopFailure',
				'SubagentStart',
				'SubagentStop',
				'TaskCompleted',
				'TeammateIdle',
				'PreCompact',
				'PostCompact',
				'ConfigChange',
				'CwdChanged',
				'FileChanged',
				'WorktreeCreate',
				'WorktreeRemove',
				'Elicitation',
				'ElicitationResult',
				'SessionEnd'
			];

			expect(validEventTypes).toHaveLength(24);
		});

		it('should define valid HookType values', () => {
			const validHookTypes: HookType[] = ['command', 'prompt'];

			// TypeScript compile-time check - if this compiles, the types are correct
			expect(validHookTypes).toHaveLength(2);
		});
	});
});
