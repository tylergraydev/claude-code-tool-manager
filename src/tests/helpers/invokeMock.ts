import { vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

/**
 * Set up command-routing mock for Tauri invoke.
 * Instead of sequential mockResolvedValueOnce chains, provide a map of
 * command names to return values. Unknown commands return undefined.
 *
 * Usage:
 *   mockInvokeResponses({
 *     'get_all_mcps': [createMockMcp()],
 *     'get_all_projects': [],
 *   });
 */
export function mockInvokeResponses(responses: Record<string, unknown>): void {
	vi.mocked(invoke).mockImplementation(async (cmd: string) => {
		if (cmd in responses) {
			const value = responses[cmd];
			// If the value is a function, call it (supports dynamic responses)
			if (typeof value === 'function') {
				return (value as () => unknown)();
			}
			return value;
		}
		return undefined;
	});
}

/**
 * Set up a command-routing mock that can also inspect arguments.
 * The handler receives (command, args) and can return different values.
 */
export function mockInvokeHandler(
	handler: (cmd: string, args?: Record<string, unknown>) => unknown
): void {
	vi.mocked(invoke).mockImplementation(async (cmd: string, args?: Record<string, unknown>) => {
		return handler(cmd, args);
	});
}
