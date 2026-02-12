import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock @tauri-apps/api/app
vi.mock('@tauri-apps/api/app', () => ({
	getVersion: vi.fn()
}));

import { getVersion } from '@tauri-apps/api/app';

describe('WhatsNew Store', () => {
	let mockLocalStorage: Record<string, string>;

	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();

		// Mock localStorage
		mockLocalStorage = {};
		vi.stubGlobal('localStorage', {
			getItem: vi.fn((key: string) => mockLocalStorage[key] ?? null),
			setItem: vi.fn((key: string, value: string) => {
				mockLocalStorage[key] = value;
			}),
			removeItem: vi.fn((key: string) => {
				delete mockLocalStorage[key];
			})
		});

		// Mock fetch
		vi.stubGlobal('fetch', vi.fn());
	});

	describe('checkForWhatsNew', () => {
		it('should save version on first run without showing modal', async () => {
			vi.mocked(getVersion).mockResolvedValueOnce('1.0.0');
			// No lastSeenVersion in localStorage (first run)

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(localStorage.setItem).toHaveBeenCalledWith(
				'claude-tool-manager-last-seen-version',
				'1.0.0'
			);
			expect(whatsNew.isOpen).toBe(false);
		});

		it('should not show modal when version unchanged', async () => {
			vi.mocked(getVersion).mockResolvedValueOnce('1.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(whatsNew.isOpen).toBe(false);
		});

		it('should fetch release notes and open modal on version change', async () => {
			vi.mocked(getVersion).mockResolvedValueOnce('2.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			const mockResponse = {
				ok: true,
				json: vi.fn().mockResolvedValue({
					tag_name: 'v2.0.0',
					name: 'Version 2.0.0',
					body: 'New features!',
					published_at: '2024-01-15T00:00:00Z',
					html_url: 'https://github.com/example/releases/v2.0.0'
				})
			};
			vi.mocked(fetch).mockResolvedValueOnce(mockResponse as any);

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(whatsNew.isOpen).toBe(true);
			expect(whatsNew.release?.version).toBe('2.0.0');
			expect(whatsNew.release?.body).toBe('New features!');
		});

		it('should try without v prefix if first fetch fails', async () => {
			vi.mocked(getVersion).mockResolvedValueOnce('2.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			const failedResponse = { ok: false };
			const successResponse = {
				ok: true,
				json: vi.fn().mockResolvedValue({
					tag_name: '2.0.0',
					name: 'Release 2.0.0',
					body: 'Changes',
					published_at: '2024-01-15T00:00:00Z',
					html_url: 'https://github.com/example/releases/2.0.0'
				})
			};

			vi.mocked(fetch)
				.mockResolvedValueOnce(failedResponse as any)
				.mockResolvedValueOnce(successResponse as any);

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(whatsNew.isOpen).toBe(true);
			expect(whatsNew.release?.version).toBe('2.0.0');
		});

		it('should show fallback release info on fetch failure', async () => {
			vi.mocked(getVersion).mockResolvedValueOnce('2.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			const failedResponse = { ok: false };
			vi.mocked(fetch)
				.mockResolvedValueOnce(failedResponse as any)
				.mockResolvedValueOnce(failedResponse as any);

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(whatsNew.isOpen).toBe(true);
			expect(whatsNew.release?.version).toBe('2.0.0');
			expect(whatsNew.error).toContain('Release not found');
		});
	});

	describe('dismiss', () => {
		it('should close modal and save version', async () => {
			vi.mocked(getVersion).mockResolvedValueOnce('2.0.0');
			mockLocalStorage['claude-tool-manager-last-seen-version'] = '1.0.0';

			const mockResponse = {
				ok: true,
				json: vi.fn().mockResolvedValue({
					tag_name: 'v2.0.0',
					name: 'v2.0.0',
					body: 'Notes',
					published_at: '2024-01-15T00:00:00Z',
					html_url: 'https://github.com/example/releases/v2.0.0'
				})
			};
			vi.mocked(fetch).mockResolvedValueOnce(mockResponse as any);

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			await whatsNew.checkForWhatsNew();

			expect(whatsNew.isOpen).toBe(true);

			whatsNew.dismiss();

			expect(whatsNew.isOpen).toBe(false);
			expect(localStorage.setItem).toHaveBeenCalledWith(
				'claude-tool-manager-last-seen-version',
				'2.0.0'
			);
		});
	});

	describe('fetchReleaseNotes', () => {
		it('should set isLoading while fetching', async () => {
			let resolveResponse: (value: unknown) => void;
			const responsePromise = new Promise((resolve) => {
				resolveResponse = resolve;
			});
			vi.mocked(fetch).mockReturnValueOnce(responsePromise as any);

			const { whatsNew } = await import('$lib/stores/whatsNew.svelte');
			const fetchPromise = whatsNew.fetchReleaseNotes('1.0.0');

			expect(whatsNew.isLoading).toBe(true);

			resolveResponse!({
				ok: true,
				json: () =>
					Promise.resolve({
						tag_name: 'v1.0.0',
						name: 'v1.0.0',
						body: 'Notes',
						published_at: '2024-01-01T00:00:00Z',
						html_url: 'https://example.com'
					})
			});
			await fetchPromise;

			expect(whatsNew.isLoading).toBe(false);
		});
	});
});
