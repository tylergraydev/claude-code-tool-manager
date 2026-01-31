import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock getVersion
vi.mock('@tauri-apps/api/app', () => ({
	getVersion: vi.fn()
}));

// Mock fetch globally
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('WhatsNew Store', () => {
	const mockLocalStorage: Record<string, string> = {};

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
		
		// Mock localStorage
		Object.defineProperty(global, 'localStorage', {
			value: {
				getItem: vi.fn((key: string) => mockLocalStorage[key] ?? null),
				setItem: vi.fn((key: string, value: string) => {
					mockLocalStorage[key] = value;
				}),
				removeItem: vi.fn((key: string) => {
					delete mockLocalStorage[key];
				}),
				clear: vi.fn()
			},
			writable: true
		});

		// Clear mock storage
		Object.keys(mockLocalStorage).forEach(key => delete mockLocalStorage[key]);
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');

			expect(whatsNew.isOpen).toBe(false);
			expect(whatsNew.isLoading).toBe(false);
			expect(whatsNew.release).toBeNull();
			expect(whatsNew.error).toBeNull();
		});
	});

	describe('checkForWhatsNew', () => {
		it('should save version on first run without showing modal', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockResolvedValueOnce('1.0.0');

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(localStorage.setItem).toHaveBeenCalledWith(
				'claude-tool-manager-last-seen-version',
				'1.0.0'
			);
			expect(whatsNew.isOpen).toBe(false);
		});

		it('should not show modal when version is the same', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockResolvedValueOnce('1.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(whatsNew.isOpen).toBe(false);
			expect(mockFetch).not.toHaveBeenCalled();
		});

		it('should fetch release notes and show modal when version changes', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockResolvedValueOnce('2.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({
					tag_name: 'v2.0.0',
					name: 'Version 2.0.0',
					body: 'New features!',
					published_at: '2024-01-01T00:00:00Z',
					html_url: 'https://github.com/example/releases/v2.0.0'
				})
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(mockFetch).toHaveBeenCalledWith(
				'https://api.github.com/repos/tylergraydev/claude-code-tool-manager/releases/tags/v2.0.0'
			);
			expect(whatsNew.isOpen).toBe(true);
			expect(whatsNew.release?.version).toBe('2.0.0');
		});

		it('should handle errors silently', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockRejectedValueOnce(new Error('Version error'));

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			// Should not throw
			expect(whatsNew.isOpen).toBe(false);
		});
	});

	describe('fetchReleaseNotes', () => {
		it('should fetch release notes with v prefix', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({
					tag_name: 'v1.5.0',
					name: 'Release 1.5.0',
					body: 'Bug fixes',
					published_at: '2024-01-01T00:00:00Z',
					html_url: 'https://github.com/example/releases/v1.5.0'
				})
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.fetchReleaseNotes('1.5.0');

			expect(whatsNew.release).toEqual({
				version: '1.5.0',
				name: 'Release 1.5.0',
				body: 'Bug fixes',
				publishedAt: '2024-01-01T00:00:00Z',
				htmlUrl: 'https://github.com/example/releases/v1.5.0'
			});
			expect(whatsNew.isLoading).toBe(false);
		});

		it('should try without v prefix if first request fails', async () => {
			mockFetch
				.mockResolvedValueOnce({ ok: false })
				.mockResolvedValueOnce({
					ok: true,
					json: () => Promise.resolve({
						tag_name: '1.5.0',
						name: 'Release 1.5.0',
						body: 'Bug fixes',
						published_at: '2024-01-01T00:00:00Z',
						html_url: 'https://github.com/example/releases/1.5.0'
					})
				});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.fetchReleaseNotes('1.5.0');

			expect(mockFetch).toHaveBeenCalledTimes(2);
			expect(whatsNew.release?.version).toBe('1.5.0');
		});

		it('should use fallback release info on error', async () => {
			mockFetch
				.mockResolvedValueOnce({ ok: false })
				.mockResolvedValueOnce({ ok: false });

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.fetchReleaseNotes('1.5.0');

			expect(whatsNew.error).toBe('Release not found');
			expect(whatsNew.release).not.toBeNull();
			expect(whatsNew.release?.version).toBe('1.5.0');
			expect(whatsNew.release?.name).toBe('Version 1.5.0');
		});

		it('should set isLoading during fetch', async () => {
			let resolveJson: () => void;
			const jsonPromise = new Promise<void>((resolve) => {
				resolveJson = resolve;
			});

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => jsonPromise.then(() => ({
					tag_name: 'v1.0.0',
					name: 'Test',
					body: 'Test',
					published_at: '2024-01-01',
					html_url: 'https://example.com'
				}))
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			const fetchPromise = whatsNew.fetchReleaseNotes('1.0.0');

			expect(whatsNew.isLoading).toBe(true);

			resolveJson!();
			await fetchPromise;

			expect(whatsNew.isLoading).toBe(false);
		});

		it('should handle missing release fields', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({
					tag_name: 'v1.5.0',
					name: null,
					body: null,
					published_at: '2024-01-01T00:00:00Z',
					html_url: 'https://example.com'
				})
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.fetchReleaseNotes('1.5.0');

			expect(whatsNew.release?.name).toBe('Version v1.5.0');
			expect(whatsNew.release?.body).toBe('No release notes available.');
		});

		it('should strip v prefix from version', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({
					tag_name: 'v2.0.0',
					name: 'Test',
					body: 'Test',
					published_at: '2024-01-01',
					html_url: 'https://example.com'
				})
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.fetchReleaseNotes('2.0.0');

			expect(whatsNew.release?.version).toBe('2.0.0');
		});
	});

	describe('dismiss', () => {
		it('should close modal and save version', async () => {
			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({
					tag_name: 'v1.5.0',
					name: 'Release',
					body: 'Notes',
					published_at: '2024-01-01',
					html_url: 'https://example.com'
				})
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.fetchReleaseNotes('1.5.0');
			whatsNew.isOpen = true;

			whatsNew.dismiss();

			expect(whatsNew.isOpen).toBe(false);
			expect(localStorage.setItem).toHaveBeenCalledWith(
				'claude-tool-manager-last-seen-version',
				'1.5.0'
			);
		});

		it('should not save version if no release', async () => {
			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			whatsNew.isOpen = true;

			whatsNew.dismiss();

			expect(whatsNew.isOpen).toBe(false);
			// Should not have called setItem (only getItem is called on load)
			expect(localStorage.setItem).not.toHaveBeenCalled();
		});
	});

	describe('showCurrentReleaseNotes', () => {
		it('should fetch and show current version notes', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockResolvedValueOnce('1.5.0');

			mockFetch.mockResolvedValueOnce({
				ok: true,
				json: () => Promise.resolve({
					tag_name: 'v1.5.0',
					name: 'Release 1.5.0',
					body: 'Notes',
					published_at: '2024-01-01',
					html_url: 'https://example.com'
				})
			});

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.showCurrentReleaseNotes();

			expect(getVersion).toHaveBeenCalled();
			expect(whatsNew.isOpen).toBe(true);
			expect(whatsNew.release?.version).toBe('1.5.0');
		});

		it('should handle errors silently', async () => {
			const { getVersion } = await import('@tauri-apps/api/app');
			vi.mocked(getVersion).mockRejectedValueOnce(new Error('Version error'));

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.showCurrentReleaseNotes();

			// Should not throw
			expect(whatsNew.isOpen).toBe(false);
		});
	});
});
