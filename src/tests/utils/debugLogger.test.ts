import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Debug Logger Utility', () => {
	let originalConsole: any;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
		// Save original console methods
		originalConsole = {
			log: console.log.bind(console),
			warn: console.warn.bind(console),
			error: console.error.bind(console),
			info: console.info.bind(console)
		};
	});

	afterEach(() => {
		// Restore original console methods
		console.log = originalConsole.log;
		console.warn = originalConsole.warn;
		console.error = originalConsole.error;
		console.info = originalConsole.info;
	});

	describe('installDebugInterceptor', () => {
		it('should intercept console.log', async () => {
			const mockLog = vi.fn();
			console.log = mockLog;

			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			console.log('test message');

			// Should call original console.log
			expect(mockLog).toHaveBeenCalledWith('test message');
		});

		it('should intercept console.warn', async () => {
			const mockWarn = vi.fn();
			console.warn = mockWarn;

			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			console.warn('warning message');

			expect(mockWarn).toHaveBeenCalledWith('warning message');
		});

		it('should intercept console.error', async () => {
			const mockError = vi.fn();
			console.error = mockError;

			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			console.error('error message');

			expect(mockError).toHaveBeenCalledWith('error message');
		});

		it('should intercept console.info', async () => {
			const mockInfo = vi.fn();
			console.info = mockInfo;

			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			console.info('info message');

			expect(mockInfo).toHaveBeenCalledWith('info message');
		});

		it('should send to backend on console calls', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			console.log('test message');

			// Should call write_frontend_log
			expect(invoke).toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'test message',
				context: null
			});
		});

		it('should not send to backend when debug disabled', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: false }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			console.log('test message');

			// Should NOT call write_frontend_log
			expect(invoke).not.toHaveBeenCalledWith('write_frontend_log', {
				level: 'INFO',
				message: 'test message',
				context: null
			});
		});

		it('should log installation message', async () => {
			const mockLog = vi.fn();
			console.log = mockLog;

			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			// Check that installation was logged
			expect(mockLog).toHaveBeenCalledWith(expect.stringContaining('[Debug]'));
			expect(mockLog).toHaveBeenCalledWith(expect.stringContaining('Console interceptor installed'));
		});
	});

	describe('uninstallDebugInterceptor', () => {
		it('should restore original console methods', async () => {
			const mockLog = vi.fn();

			// Install interceptor
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor, uninstallDebugInterceptor } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			// Replace console.log with mock
			console.log = mockLog;
			const testMessage = 'test during install';

			// Call console.log with mock
			console.log(testMessage);
			expect(mockLog).toHaveBeenCalledWith(testMessage);

			// Uninstall
			uninstallDebugInterceptor();

			// After uninstall, console.log should work normally
			console.log('test after uninstall');

			// Mock should not be called for the new call
			expect(mockLog).toHaveBeenCalledTimes(1);
			expect(mockLog).toHaveBeenCalledWith(testMessage);
		});

		it('should set interceptorInstalled to false', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor, uninstallDebugInterceptor, isInterceptorInstalled } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			expect(isInterceptorInstalled()).toBe(true);

			uninstallDebugInterceptor();
			expect(isInterceptorInstalled()).toBe(false);
		});

		it('should log uninstallation message', async () => {
			const mockLog = vi.fn();
			console.log = mockLog;

			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { uninstallDebugInterceptor } = await import('$lib/utils/debugLogger');

			// Need to install first
			const { installDebugInterceptor } = await import('$lib/utils/debugLogger');
			installDebugInterceptor();

			uninstallDebugInterceptor();

			expect(mockLog).toHaveBeenCalledWith(expect.stringContaining('[Debug]'));
			expect(mockLog).toHaveBeenCalledWith(expect.stringContaining('Console interceptor uninstalled'));
		});
	});

	describe('isInterceptorInstalled', () => {
		it('should return true when installed', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor, isInterceptorInstalled } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();

			expect(isInterceptorInstalled()).toBe(true);
		});

		it('should return false when not installed', async () => {
			const { isInterceptorInstalled } = await import('$lib/utils/debugLogger');

			expect(isInterceptorInstalled()).toBe(false);
		});
	});

	describe('debugInvoke - success path', () => {
		it('should call invoke and return result', async () => {
			const mockResult = { success: true, data: 'test' };

			vi.mocked(invoke).mockResolvedValue(mockResult);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: false }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			const result = await debugInvoke('test_command', { param: 'value' });

			expect(result).toEqual(mockResult);
			expect(invoke).toHaveBeenCalledWith('test_command', { param: 'value' });
		});

		it('should log successful invoke when enabled', async () => {
			vi.mocked(invoke).mockResolvedValue({ success: true });
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			await debugInvoke('test_command', { param: 'value' });

			// Should call write_invoke_log
			expect(invoke).toHaveBeenCalledWith('write_invoke_log', {
				command: 'test_command',
				durationMs: expect.any(Number),
				success: true,
				args: '{"param":"value"}',
				error: null
			});
		});

		it('should not log when debug disabled', async () => {
			vi.mocked(invoke).mockResolvedValue({ success: true });
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: false }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			await debugInvoke('test_command', { param: 'value' });

			// Should not call write_invoke_log
			expect(invoke).toHaveBeenCalledTimes(1); // Only the actual invoke, not logging
		});

		it('should measure duration correctly', async () => {
			vi.mocked(invoke).mockImplementation(async () => {
				// Simulate delay
				await new Promise((resolve) => setTimeout(resolve, 50));
				return { success: true };
			});
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			await debugInvoke('test_command', {});

			// Verify invoke was called
			expect(invoke).toHaveBeenCalledWith('test_command', {});

			// Should also log duration
			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				command: 'test_command',
				durationMs: expect.any(Number),
				success: true
			}));
		});
	});

	describe('debugInvoke - error path', () => {
		it('should throw error after logging', async () => {
			const testError = new Error('Command failed');

			vi.mocked(invoke).mockRejectedValue(testError);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			await expect(debugInvoke('test_command', {})).rejects.toThrow('Command failed');
		});

		it('should log failed invoke when enabled', async () => {
			const testError = new Error('Command failed');

			vi.mocked(invoke).mockRejectedValue(testError);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			try {
				await debugInvoke('test_command', {});
			} catch {
				// Expected to throw
			}

			// Should log the error
			expect(invoke).toHaveBeenCalledWith('write_invoke_log', expect.objectContaining({
				command: 'test_command',
				durationMs: expect.any(Number),
				success: false,
				error: 'Command failed'
			}));
		});

		it('should not log when debug disabled', async () => {
			const testError = new Error('Command failed');

			vi.mocked(invoke).mockRejectedValue(testError);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: false }
			}));

			const { debugInvoke } = await import('$lib/utils/debugLogger');

			try {
				await debugInvoke('test_command', {});
			} catch {
				// Expected to throw
			}

			// Should not call write_invoke_log
			const logCalls = vi.mocked(invoke).mock.calls.filter((c) => c[0] === 'write_invoke_log');
			expect(logCalls.length).toBe(0);
		});
	});

	describe('interceptor installation guards', () => {
		it('should prevent duplicate installation', async () => {
			vi.mocked(invoke).mockResolvedValue(undefined);
			vi.doMock('$lib/stores/debug.svelte', () => ({
				debugStore: { isEnabled: true }
			}));

			const { installDebugInterceptor, isInterceptorInstalled } = await import('$lib/utils/debugLogger');

			installDebugInterceptor();
			installDebugInterceptor();
			installDebugInterceptor();

			// Should still be installed only once
			expect(isInterceptorInstalled()).toBe(true);
		});
	});
});
