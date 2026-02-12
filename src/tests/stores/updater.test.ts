import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the Tauri updater plugin
vi.mock('@tauri-apps/plugin-updater', () => ({
	check: vi.fn()
}));

// Mock the Tauri process plugin
vi.mock('@tauri-apps/plugin-process', () => ({
	relaunch: vi.fn()
}));

import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

describe('Updater Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('checkForUpdates', () => {
		it('should transition from idle to checking to available when update found', async () => {
			const mockUpdate = {
				version: '2.0.0',
				downloadAndInstall: vi.fn()
			};
			vi.mocked(check).mockResolvedValueOnce(mockUpdate as any);

			const { updater } = await import('$lib/stores/updater.svelte');

			expect(updater.status).toBe('idle');
			await updater.checkForUpdates();

			expect(updater.status).toBe('available');
			expect(updater.update).toStrictEqual(mockUpdate);
		});

		it('should transition to idle when no update available', async () => {
			vi.mocked(check).mockResolvedValueOnce(null as any);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.status).toBe('idle');
			expect(updater.update).toBeNull();
		});

		it('should set error status on failure', async () => {
			vi.mocked(check).mockRejectedValueOnce(new Error('Network error'));

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.status).toBe('error');
			expect(updater.error).toBe('Network error');
		});

		it('should not check if already checking', async () => {
			let resolveCheck: (value: unknown) => void;
			const checkPromise = new Promise((resolve) => {
				resolveCheck = resolve;
			});
			vi.mocked(check).mockReturnValueOnce(checkPromise as any);

			const { updater } = await import('$lib/stores/updater.svelte');

			// Start first check
			const p1 = updater.checkForUpdates();

			// Try second check while first is pending
			await updater.checkForUpdates();

			// Should only have been called once
			expect(check).toHaveBeenCalledTimes(1);

			resolveCheck!(null);
			await p1;
		});

		it('should not check if currently downloading', async () => {
			const mockUpdate = {
				version: '2.0.0',
				downloadAndInstall: vi.fn().mockReturnValue(new Promise(() => {})) // never resolves
			};
			vi.mocked(check).mockResolvedValueOnce(mockUpdate as any);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			// Start download (won't resolve)
			updater.downloadAndInstall();

			// Try to check again
			await updater.checkForUpdates();

			// check should only have been called once
			expect(check).toHaveBeenCalledTimes(1);
		});
	});

	describe('downloadAndInstall', () => {
		it('should track progress and set ready on completion', async () => {
			const mockUpdate = {
				version: '2.0.0',
				downloadAndInstall: vi.fn().mockImplementation(async (callback: Function) => {
					callback({ event: 'Started', data: { contentLength: 1000 } });
					callback({ event: 'Progress', data: { chunkLength: 500 } });
					callback({ event: 'Finished' });
				})
			};
			vi.mocked(check).mockResolvedValueOnce(mockUpdate as any);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();
			await updater.downloadAndInstall();

			expect(updater.status).toBe('ready');
			expect(updater.downloadProgress).toBe(100);
		});

		it('should not download if no update available', async () => {
			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.downloadAndInstall();

			expect(updater.status).toBe('idle');
		});

		it('should handle download errors', async () => {
			const mockUpdate = {
				version: '2.0.0',
				downloadAndInstall: vi.fn().mockRejectedValueOnce(new Error('Download failed'))
			};
			vi.mocked(check).mockResolvedValueOnce(mockUpdate as any);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();
			await updater.downloadAndInstall();

			expect(updater.status).toBe('error');
			expect(updater.error).toBe('Download failed');
		});
	});

	describe('dismiss', () => {
		it('should reset to idle', async () => {
			const mockUpdate = {
				version: '2.0.0',
				downloadAndInstall: vi.fn()
			};
			vi.mocked(check).mockResolvedValueOnce(mockUpdate as any);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.status).toBe('available');

			updater.dismiss();

			expect(updater.status).toBe('idle');
			expect(updater.update).toBeNull();
			expect(updater.error).toBeNull();
		});
	});

	describe('restartApp', () => {
		it('should call relaunch', async () => {
			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.restartApp();

			expect(relaunch).toHaveBeenCalledOnce();
		});
	});
});
