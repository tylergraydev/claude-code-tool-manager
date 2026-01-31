import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Debug Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { debugStore } = await import('$lib/stores/debug.svelte');

			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.logFilePath).toBeNull();
			expect(debugStore.isLoading).toBe(false);
		});
	});

	describe('load', () => {
		it('should load debug state when disabled', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(false);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.load();

			expect(invoke).toHaveBeenCalledWith('is_debug_mode_enabled');
			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.logFilePath).toBeNull();
		});

		it('should load debug state and log path when enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(true)
				.mockResolvedValueOnce('/path/to/debug.log');

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.load();

			expect(invoke).toHaveBeenCalledWith('is_debug_mode_enabled');
			expect(invoke).toHaveBeenCalledWith('get_debug_log_path');
			expect(debugStore.isEnabled).toBe(true);
			expect(debugStore.logFilePath).toBe('/path/to/debug.log');
		});

		it('should handle errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.load();

			// Should not throw, just log the error
			expect(debugStore.isEnabled).toBe(false);
		});
	});

	describe('enable', () => {
		it('should enable debug mode and set log path', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('/path/to/debug.log');

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			expect(invoke).toHaveBeenCalledWith('enable_debug_mode');
			expect(debugStore.isEnabled).toBe(true);
			expect(debugStore.logFilePath).toBe('/path/to/debug.log');
			expect(debugStore.isLoading).toBe(false);
		});

		it('should set isLoading during enable', async () => {
			let resolvePromise: (value: string) => void;
			const promise = new Promise<string>((resolve) => {
				resolvePromise = resolve;
			});

			vi.mocked(invoke).mockReturnValueOnce(promise);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			const enablePromise = debugStore.enable();

			expect(debugStore.isLoading).toBe(true);

			resolvePromise!('/path/to/log');
			await enablePromise;

			expect(debugStore.isLoading).toBe(false);
		});

		it('should handle errors and rethrow', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Enable failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');

			await expect(debugStore.enable()).rejects.toThrow('Enable failed');
			expect(debugStore.isLoading).toBe(false);
		});
	});

	describe('disable', () => {
		it('should disable debug mode', async () => {
			// First enable
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();
			expect(debugStore.isEnabled).toBe(true);

			await debugStore.disable();

			expect(invoke).toHaveBeenCalledWith('disable_debug_mode');
			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.isLoading).toBe(false);
		});

		it('should set isLoading during disable', async () => {
			let resolvePromise: (value: undefined) => void;
			const promise = new Promise<undefined>((resolve) => {
				resolvePromise = resolve;
			});

			vi.mocked(invoke).mockReturnValueOnce(promise);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			const disablePromise = debugStore.disable();

			expect(debugStore.isLoading).toBe(true);

			resolvePromise!(undefined);
			await disablePromise;

			expect(debugStore.isLoading).toBe(false);
		});

		it('should handle errors and rethrow', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Disable failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');

			await expect(debugStore.disable()).rejects.toThrow('Disable failed');
			expect(debugStore.isLoading).toBe(false);
		});
	});

	describe('toggle', () => {
		it('should enable when disabled', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('/path/to/log');

			const { debugStore } = await import('$lib/stores/debug.svelte');
			expect(debugStore.isEnabled).toBe(false);

			await debugStore.toggle();

			expect(debugStore.isEnabled).toBe(true);
		});

		it('should disable when enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();
			expect(debugStore.isEnabled).toBe(true);

			await debugStore.toggle();

			expect(debugStore.isEnabled).toBe(false);
		});
	});

	describe('openLogsFolder', () => {
		it('should call invoke to open logs folder', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.openLogsFolder();

			expect(invoke).toHaveBeenCalledWith('open_logs_folder');
		});

		it('should handle errors and rethrow', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Open failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');

			await expect(debugStore.openLogsFolder()).rejects.toThrow('Open failed');
		});
	});

	describe('log', () => {
		it('should not log when debug is disabled', async () => {
			const { debugStore } = await import('$lib/stores/debug.svelte');
			expect(debugStore.isEnabled).toBe(false);

			await debugStore.log('Test message');

			expect(invoke).not.toHaveBeenCalled();
		});

		it('should log when debug is enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			await debugStore.log('Test message', 'TestContext');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'Test message',
				context: 'TestContext'
			});
		});

		it('should handle null context', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			await debugStore.log('Test message');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'Test message',
				context: null
			});
		});

		it('should silently fail on error', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockRejectedValueOnce(new Error('Log failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			// Should not throw
			await debugStore.log('Test message');
		});
	});

	describe('warn', () => {
		it('should not warn when debug is disabled', async () => {
			const { debugStore } = await import('$lib/stores/debug.svelte');

			await debugStore.warn('Warning message');

			expect(invoke).not.toHaveBeenCalled();
		});

		it('should warn when debug is enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			await debugStore.warn('Warning message', 'WarnContext');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'WARN',
				message: 'Warning message',
				context: 'WarnContext'
			});
		});
	});

	describe('error', () => {
		it('should not log error when debug is disabled', async () => {
			const { debugStore } = await import('$lib/stores/debug.svelte');

			await debugStore.error('Error message');

			expect(invoke).not.toHaveBeenCalled();
		});

		it('should log error when debug is enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			await debugStore.error('Error message', 'ErrorContext');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'ERROR',
				message: 'Error message',
				context: 'ErrorContext'
			});
		});
	});

	describe('logInvoke', () => {
		it('should not log invoke when debug is disabled', async () => {
			const { debugStore } = await import('$lib/stores/debug.svelte');

			await debugStore.logInvoke('test_command', 100, true);

			expect(invoke).not.toHaveBeenCalled();
		});

		it('should log successful invoke when debug is enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			await debugStore.logInvoke('test_command', 150, true, { arg1: 'value' });

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', {
				command: 'test_command',
				durationMs: 150,
				success: true,
				args: JSON.stringify({ arg1: 'value' }),
				error: null
			});
		});

		it('should log failed invoke with error', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			await debugStore.logInvoke('test_command', 50, false, undefined, 'Command failed');

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', {
				command: 'test_command',
				durationMs: 50,
				success: false,
				args: null,
				error: 'Command failed'
			});
		});

		it('should silently fail on error', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('/path/to/log')
				.mockRejectedValueOnce(new Error('Log failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			// Should not throw
			await debugStore.logInvoke('test_command', 100, true);
		});
	});
});
