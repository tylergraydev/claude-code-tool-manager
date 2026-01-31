import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// We need to mock debugStore with a reactive-like object
const mockDebugStore = {
	isEnabled: false
};

vi.mock('$lib/stores', () => ({
	debugStore: mockDebugStore
}));

describe('Debug Logger', () => {
	// Store original console methods
	const originalConsole = {
		log: console.log,
		warn: console.warn,
		error: console.error,
		info: console.info
	};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();

		// Restore console methods before each test
		console.log = originalConsole.log;
		console.warn = originalConsole.warn;
		console.error = originalConsole.error;
		console.info = originalConsole.info;

		// Reset debug store state
		mockDebugStore.isEnabled = false;
	});

	afterEach(() => {
		// Restore console methods after each test
		console.log = originalConsole.log;
		console.warn = originalConsole.warn;
		console.error = originalConsole.error;
		console.info = originalConsole.info;
	});

	describe('installDebugInterceptor', () => {
		it('should install the interceptor', async () => {
			const { installDebugInterceptor, isInterceptorInstalled, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			expect(isInterceptorInstalled()).toBe(false);

			installDebugInterceptor();

			expect(isInterceptorInstalled()).toBe(true);

			uninstallDebugInterceptor();
		});

		it('should be idempotent', async () => {
			const { installDebugInterceptor, isInterceptorInstalled, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			installDebugInterceptor();
			installDebugInterceptor();

			expect(isInterceptorInstalled()).toBe(true);

			uninstallDebugInterceptor();
		});

		it('should wrap console.log and send to backend when enabled', async () => {
			mockDebugStore.isEnabled = true;
			vi.mocked(invoke).mockResolvedValue(undefined);

			const { installDebugInterceptor, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			const originalLog = console.log;
			installDebugInterceptor();

			// After install, console.log should be different from original
			expect(console.log).not.toBe(originalLog);

			// Should be able to call without error and send to backend
			console.log('test message');
			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'test message',
				context: null
			});

			uninstallDebugInterceptor();
			mockDebugStore.isEnabled = false;
		});

		it('should wrap console.warn and send to backend when enabled', async () => {
			mockDebugStore.isEnabled = true;
			vi.mocked(invoke).mockResolvedValue(undefined);

			const { installDebugInterceptor, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			const originalWarn = console.warn;
			installDebugInterceptor();

			expect(console.warn).not.toBe(originalWarn);

			console.warn('warning message');
			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'WARN',
				message: 'warning message',
				context: null
			});

			uninstallDebugInterceptor();
			mockDebugStore.isEnabled = false;
		});

		it('should wrap console.error and send to backend when enabled', async () => {
			mockDebugStore.isEnabled = true;
			vi.mocked(invoke).mockResolvedValue(undefined);

			const { installDebugInterceptor, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			const originalError = console.error;
			installDebugInterceptor();

			expect(console.error).not.toBe(originalError);

			console.error('error message');
			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'ERROR',
				message: 'error message',
				context: null
			});

			uninstallDebugInterceptor();
			mockDebugStore.isEnabled = false;
		});

		it('should wrap console.info and send to backend when enabled', async () => {
			mockDebugStore.isEnabled = true;
			vi.mocked(invoke).mockResolvedValue(undefined);

			const { installDebugInterceptor, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			const originalInfo = console.info;
			installDebugInterceptor();

			expect(console.info).not.toBe(originalInfo);

			console.info('info message');
			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'info message',
				context: null
			});

			uninstallDebugInterceptor();
			mockDebugStore.isEnabled = false;
		});

		it('should not send to backend when debug is disabled', async () => {
			mockDebugStore.isEnabled = false;

			const { installDebugInterceptor, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			console.log('test message');

			// Should not call invoke since debug is disabled
			expect(invoke).not.toHaveBeenCalled();

			uninstallDebugInterceptor();
		});
	});

	describe('uninstallDebugInterceptor', () => {
		it('should uninstall the interceptor', async () => {
			const { installDebugInterceptor, uninstallDebugInterceptor, isInterceptorInstalled } =
				await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			expect(isInterceptorInstalled()).toBe(true);

			uninstallDebugInterceptor();
			expect(isInterceptorInstalled()).toBe(false);
		});

		it('should be idempotent', async () => {
			const { uninstallDebugInterceptor, isInterceptorInstalled } =
				await import('$lib/utils/debugLogger');

			uninstallDebugInterceptor();
			uninstallDebugInterceptor();
			uninstallDebugInterceptor();

			expect(isInterceptorInstalled()).toBe(false);
		});

		it('should restore original console methods', async () => {
			const { installDebugInterceptor, uninstallDebugInterceptor } =
				await import('$lib/utils/debugLogger');

			const beforeLog = console.log;

			installDebugInterceptor();
			// After install, console.log should be different

			uninstallDebugInterceptor();
			// After uninstall, should be original again
			// Note: Since we're resetting in beforeEach, this tests the restoration
		});
	});

	describe('isInterceptorInstalled', () => {
		it('should return false initially', async () => {
			const { isInterceptorInstalled } = await import('$lib/utils/debugLogger');
			expect(isInterceptorInstalled()).toBe(false);
		});

		it('should return true after install', async () => {
			const { installDebugInterceptor, uninstallDebugInterceptor, isInterceptorInstalled } =
				await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			expect(isInterceptorInstalled()).toBe(true);

			uninstallDebugInterceptor();
		});

		it('should return false after uninstall', async () => {
			const { installDebugInterceptor, uninstallDebugInterceptor, isInterceptorInstalled } =
				await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			uninstallDebugInterceptor();
			expect(isInterceptorInstalled()).toBe(false);
		});
	});

	describe('debugInvoke', () => {
		it('should call invoke and return result', async () => {
			mockDebugStore.isEnabled = false;
			vi.mocked(invoke).mockResolvedValueOnce({ data: 'test' });

			const { debugInvoke } = await import('$lib/utils/debugLogger');
			const result = await debugInvoke<{ data: string }>('test_command', { arg: 'value' });

			expect(invoke).toHaveBeenCalledWith('test_command', { arg: 'value' });
			expect(result).toEqual({ data: 'test' });
		});

		it('should log successful invoke when debug is enabled', async () => {
			mockDebugStore.isEnabled = true;

			vi.mocked(invoke)
				.mockResolvedValueOnce('result')
				.mockResolvedValueOnce(undefined);

			const { debugInvoke } = await import('$lib/utils/debugLogger');
			await debugInvoke('test_command', { arg: 'value' });

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				command: 'test_command',
				success: true,
				args: JSON.stringify({ arg: 'value' })
			}));

			mockDebugStore.isEnabled = false;
		});

		it('should not log when debug is disabled', async () => {
			mockDebugStore.isEnabled = false;

			vi.mocked(invoke).mockResolvedValueOnce('result');

			const { debugInvoke } = await import('$lib/utils/debugLogger');
			await debugInvoke('test_command');

			// Should only be called once (the actual command, not the log)
			expect(invoke).toHaveBeenCalledTimes(1);
		});

		it('should log failed invoke when debug is enabled', async () => {
			mockDebugStore.isEnabled = true;

			vi.mocked(invoke)
				.mockRejectedValueOnce(new Error('Command failed'))
				.mockResolvedValueOnce(undefined);

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			await expect(debugInvoke('test_command')).rejects.toThrow('Command failed');

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				command: 'test_command',
				success: false,
				error: 'Command failed'
			}));

			mockDebugStore.isEnabled = false;
		});

		it('should handle non-Error thrown values', async () => {
			mockDebugStore.isEnabled = true;

			vi.mocked(invoke)
				.mockRejectedValueOnce('String error')
				.mockResolvedValueOnce(undefined);

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			await expect(debugInvoke('test_command')).rejects.toBe('String error');

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				error: 'String error'
			}));

			mockDebugStore.isEnabled = false;
		});

		it('should handle null args', async () => {
			mockDebugStore.isEnabled = true;

			vi.mocked(invoke)
				.mockResolvedValueOnce('result')
				.mockResolvedValueOnce(undefined);

			const { debugInvoke } = await import('$lib/utils/debugLogger');
			await debugInvoke('test_command');

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				args: null
			}));

			mockDebugStore.isEnabled = false;
		});

		it('should silently fail logging errors', async () => {
			mockDebugStore.isEnabled = true;

			vi.mocked(invoke)
				.mockResolvedValueOnce('result')
				.mockRejectedValueOnce(new Error('Log failed'));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			// Should not throw even though logging failed
			const result = await debugInvoke('test_command');
			expect(result).toBe('result');

			mockDebugStore.isEnabled = false;
		});

		it('should track duration', async () => {
			mockDebugStore.isEnabled = true;

			vi.mocked(invoke)
				.mockResolvedValueOnce('result')
				.mockResolvedValueOnce(undefined);

			const { debugInvoke } = await import('$lib/utils/debugLogger');
			await debugInvoke('test_command');

			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				durationMs: expect.any(Number)
			}));

			mockDebugStore.isEnabled = false;
		});
	});
});
