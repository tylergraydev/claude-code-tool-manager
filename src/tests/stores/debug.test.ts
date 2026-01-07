import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Debug Store', () => {
	beforeEach(() => {
		vi.resetModules();
		vi.clearAllMocks();
	});

	describe('load', () => {
		it('should load debug state', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(true)
				.mockResolvedValueOnce('/path/to/logs');

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.load();

			expect(debugStore.isEnabled).toBe(true);
			expect(debugStore.logFilePath).toBe('/path/to/logs');
		});

		it('should set log file path when enabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(true)
				.mockResolvedValueOnce(null);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.load();

			expect(debugStore.logFilePath).toBeNull();
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.load();

			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.logFilePath).toBeNull();
		});
	});

	describe('enable', () => {
		it('should enable debug mode', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('/path/to/logs');

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.enable();

			expect(debugStore.isEnabled).toBe(true);
			expect(debugStore.logFilePath).toBe('/path/to/logs');
			expect(debugStore.isLoading).toBe(false);
		});

		it('should set isLoading during enable', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});

			vi.mocked(invoke).mockReturnValueOnce(invokePromise as any);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			const enablePromise = debugStore.enable();

			expect(debugStore.isLoading).toBe(true);

			resolveInvoke!('/path/to/logs');

			await enablePromise;

			expect(debugStore.isLoading).toBe(false);
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Enable failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await expect(debugStore.enable()).rejects.toThrow('Enable failed');

			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.logFilePath).toBeNull();
			expect(debugStore.isLoading).toBe(false);
		});
	});

	describe('disable', () => {
		it('should disable debug mode', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.logFilePath = '/path/to/logs';

			await debugStore.disable();

			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.logFilePath).toBe('/path/to/logs');
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Disable failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.logFilePath = '/path/to/logs';

			await expect(debugStore.disable()).rejects.toThrow('Disable failed');

			expect(debugStore.isEnabled).toBe(false);
		});
	});

	describe('toggle', () => {
		it('should enable if currently disabled', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('/path/to/logs');

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = false;

			await debugStore.toggle();

			expect(debugStore.isEnabled).toBe(true);
			expect(debugStore.logFilePath).toBe('/path/to/logs');
		});

		it('should disable if currently enabled', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;
			debugStore.logFilePath = '/path/to/logs';

			await debugStore.toggle();

			expect(debugStore.isEnabled).toBe(false);
			expect(debugStore.logFilePath).toBe('/path/to/logs');
		});
	});

	describe('openLogsFolder', () => {
		it('should open logs folder', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			await debugStore.openLogsFolder();

			expect(invoke).toHaveBeenCalledWith('open_logs_folder');
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Folder not found'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;
			debugStore.logFilePath = '/path/to/logs';

			await expect(debugStore.openLogsFolder()).rejects.toThrow('Folder not found');

			expect(debugStore.isEnabled).toBe(true);
			expect(debugStore.logFilePath).toBe('/path/to/logs');
		});
	});

	describe('log methods', () => {
		it('should log INFO message', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;

			await debugStore.log('Test message', 'context');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'Test message',
				context: 'context'
			});
		});

		it('should log INFO when disabled', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = false;

			await debugStore.log('Test message');

			expect(invoke).not.toHaveBeenCalled();
		});

		it('should log WARN message', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;

			await debugStore.warn('Warning message', 'context');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'WARN',
				message: 'Warning message',
				context: 'context'
			});
		});

		it('should log ERROR message', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;

			await debugStore.error('Error message', 'context');

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'ERROR',
				message: 'Error message',
				context: 'context'
			});
		});

		it('should handle invoke errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Write failed'));

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;

			try {
				await debugStore.log('Test message');
			} catch {
			}

			expect(invoke).toHaveBeenCalledWith('write_frontend_log', expect.objectContaining({
				level: 'INFO',
				message: 'Test message'
			}));
		});
	});

	describe('logInvoke', () => {
		it('should log successful invoke', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('result')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = true;

			await debugStore.logInvoke('test_command', 100, true, { param: 'value' });

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				command: 'test_command',
				durationMs: 100,
				success: true,
				args: '{"param":"value"}',
				error: null
			}));
		});

		it('should log invoke when disabled', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce('result')
				.mockResolvedValueOnce(undefined);

			const { debugStore } = await import('$lib/stores/debug.svelte');
			debugStore.isEnabled = false;

			await debugStore.logInvoke('test_command', 50, true, { param: 'value' });

			const logCalls = vi.mocked(invoke).mock.calls.filter((c) => c[0] === 'write_invoke_log');
			expect(logCalls.length).toBe(0);
		});
	});
});
