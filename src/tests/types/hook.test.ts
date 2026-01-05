import { describe, it, expect } from 'vitest';
import { HOOK_EVENT_TYPES } from '$lib/types/hook';
import type { HookEventType, HookType } from '$lib/types/hook';

describe('Hook Types', () => {
	describe('HOOK_EVENT_TYPES', () => {
		it('should contain all expected event types', () => {
			const eventValues = HOOK_EVENT_TYPES.map((e) => e.value);
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

			expect(preToolUse?.matcherHint).toBeDefined();
			expect(postToolUse?.matcherHint).toBeDefined();
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

		it('should have 10 event types total', () => {
			expect(HOOK_EVENT_TYPES.length).toBe(10);
		});
	});

	describe('Type definitions', () => {
		it('should define valid HookEventType values', () => {
			const validEventTypes: HookEventType[] = [
				'SessionStart',
				'UserPromptSubmit',
				'PreToolUse',
				'PermissionRequest',
				'PostToolUse',
				'Notification',
				'Stop',
				'SubagentStop',
				'PreCompact',
				'SessionEnd'
			];

			// TypeScript compile-time check - if this compiles, the types are correct
			expect(validEventTypes).toHaveLength(10);
		});

		it('should define valid HookType values', () => {
			const validHookTypes: HookType[] = ['command', 'prompt'];

			// TypeScript compile-time check - if this compiles, the types are correct
			expect(validHookTypes).toHaveLength(2);
		});
	});
});
