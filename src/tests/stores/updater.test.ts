import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Update } from '@tauri-apps/plugin-updater';

// Mock the updater plugin
vi.mock('@tauri-apps/plugin-updater', () => ({
	check: vi.fn()
}));

// Mock the process plugin
vi.mock('@tauri-apps/plugin-process', () => ({
	relaunch: vi.fn()
}));

describe('Updater Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockUpdate = (): Update => ({
		version: '2.0.0',
		date: '2024-01-01',
		body: 'New features',
		downloadAndInstall: vi.fn().mockResolvedValue(undefined)
	} as unknown as Update);

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { updater } = await import('$lib/stores/updater.svelte');

			expect(updater.status).toBe('idle');
			expect(updater.update).toBeNull();
			expect(updater.error).toBeNull();
			expect(updater.downloadProgress).toBe(0);
		});
	});

	describe('checkForUpdates', () => {
		it('should check for updates and set available status when update exists', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(check).toHaveBeenCalled();
			expect(updater.update).not.toBeNull();
			expect(updater.update?.version).toBe('2.0.0');
			expect(updater.status).toBe('available');
		});

		it('should set idle status when no update available', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			vi.mocked(check).mockResolvedValueOnce(null);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.update).toBeNull();
			expect(updater.status).toBe('idle');
		});

		it('should set error status on check failure', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			vi.mocked(check).mockRejectedValueOnce(new Error('Network error'));

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.error).toBe('Network error');
			expect(updater.status).toBe('error');
		});

		it('should handle non-Error thrown values', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			vi.mocked(check).mockRejectedValueOnce('String error');

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.error).toBe('Failed to check for updates');
			expect(updater.status).toBe('error');
		});

		it('should not check if already checking', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			let resolveCheck: (value: null) => void;
			const pendingCheck = new Promise<null>((resolve) => {
				resolveCheck = resolve;
			});
			vi.mocked(check).mockReturnValueOnce(pendingCheck);

			const { updater } = await import('$lib/stores/updater.svelte');

			// Start first check
			const firstCheck = updater.checkForUpdates();
			expect(updater.status).toBe('checking');

			// Try to start another check while first is in progress
			await updater.checkForUpdates();

			// Should only have been called once
			expect(check).toHaveBeenCalledTimes(1);

			resolveCheck!(null);
			await firstCheck;
		});

		it('should not check if currently downloading', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			
			// Make downloadAndInstall hang
			let resolveDownload: () => void;
			const pendingDownload = new Promise<void>((resolve) => {
				resolveDownload = resolve;
			});
			mockUpdate.downloadAndInstall = vi.fn().mockReturnValue(pendingDownload);
			
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			// Start download
			const downloadPromise = updater.downloadAndInstall();
			expect(updater.status).toBe('downloading');

			// Try to check for updates while downloading
			vi.mocked(check).mockClear();
			await updater.checkForUpdates();

			// Should not have called check
			expect(check).not.toHaveBeenCalled();

			resolveDownload!();
			await downloadPromise;
		});
	});

	describe('downloadAndInstall', () => {
		it('should download and install update', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();
			await updater.downloadAndInstall();

			expect(mockUpdate.downloadAndInstall).toHaveBeenCalled();
			expect(updater.status).toBe('ready');
		});

		it('should not download if no update', async () => {
			const { updater } = await import('$lib/stores/updater.svelte');

			await updater.downloadAndInstall();

			// Should remain idle (no update to download)
			expect(updater.status).toBe('idle');
		});

		it('should not download if already downloading', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			
			let resolveDownload: () => void;
			const pendingDownload = new Promise<void>((resolve) => {
				resolveDownload = resolve;
			});
			mockUpdate.downloadAndInstall = vi.fn().mockReturnValue(pendingDownload);
			
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			// Start first download
			const firstDownload = updater.downloadAndInstall();
			expect(updater.status).toBe('downloading');

			// Try second download
			await updater.downloadAndInstall();

			// Should only have been called once
			expect(mockUpdate.downloadAndInstall).toHaveBeenCalledTimes(1);

			resolveDownload!();
			await firstDownload;
		});

		it('should handle download errors', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			mockUpdate.downloadAndInstall = vi.fn().mockRejectedValue(new Error('Download failed'));
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();
			await updater.downloadAndInstall();

			expect(updater.error).toBe('Download failed');
			expect(updater.status).toBe('error');
		});

		it('should handle non-Error thrown values during download', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			mockUpdate.downloadAndInstall = vi.fn().mockRejectedValue('String error');
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();
			await updater.downloadAndInstall();

			expect(updater.error).toBe('Failed to download update');
			expect(updater.status).toBe('error');
		});

		it('should track download progress events', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			
			mockUpdate.downloadAndInstall = vi.fn().mockImplementation(async (callback) => {
				// Simulate progress events
				callback({ event: 'Started', data: { contentLength: 1000 } });
				callback({ event: 'Progress', data: { chunkLength: 50 } });
				callback({ event: 'Finished' });
			});
			
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();
			await updater.downloadAndInstall();

			expect(updater.downloadProgress).toBe(100);
			expect(updater.status).toBe('ready');
		});
	});

	describe('restartApp', () => {
		it('should call relaunch', async () => {
			const { relaunch } = await import('@tauri-apps/plugin-process');

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.restartApp();

			expect(relaunch).toHaveBeenCalled();
		});
	});

	describe('dismiss', () => {
		it('should reset all state', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			const mockUpdate = createMockUpdate();
			vi.mocked(check).mockResolvedValueOnce(mockUpdate);

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.update).not.toBeNull();
			expect(updater.status).toBe('available');

			updater.dismiss();

			expect(updater.status).toBe('idle');
			expect(updater.update).toBeNull();
			expect(updater.error).toBeNull();
		});

		it('should clear error state', async () => {
			const { check } = await import('@tauri-apps/plugin-updater');
			vi.mocked(check).mockRejectedValueOnce(new Error('Test error'));

			const { updater } = await import('$lib/stores/updater.svelte');
			await updater.checkForUpdates();

			expect(updater.error).not.toBeNull();
			expect(updater.status).toBe('error');

			updater.dismiss();

			expect(updater.error).toBeNull();
			expect(updater.status).toBe('idle');
		});
	});
});
