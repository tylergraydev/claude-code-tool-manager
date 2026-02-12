import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { createMockStatusLine, createMockGalleryEntry, resetIdCounter } from '../factories';

describe('StatusLine Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetIdCounter();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load statuslines and active statusline', async () => {
			const mockStatusLines = [
				createMockStatusLine({ id: 1, name: 'default' }),
				createMockStatusLine({ id: 2, name: 'minimal' })
			];
			const active = createMockStatusLine({ id: 1, isActive: true });

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockStatusLines) // get_all_statuslines
				.mockResolvedValueOnce(active); // get_active_statusline

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();

			expect(statuslineLibrary.statuslines).toHaveLength(2);
			expect(statuslineLibrary.activeStatusLine?.id).toBe(1);
		});

		it('should set isLoading during load', async () => {
			let resolveFirst: (value: unknown) => void;
			const firstPromise = new Promise((resolve) => {
				resolveFirst = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(firstPromise as Promise<unknown>);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			const loadPromise = statuslineLibrary.load();

			expect(statuslineLibrary.isLoading).toBe(true);

			vi.mocked(invoke).mockResolvedValueOnce(null);
			resolveFirst!([]);
			await loadPromise;

			expect(statuslineLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();

			expect(statuslineLibrary.error).toContain('Failed to load');
			expect(statuslineLibrary.isLoading).toBe(false);
		});

		it('should handle null active statusline', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(null);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();

			expect(statuslineLibrary.activeStatusLine).toBeNull();
		});
	});

	describe('filteredStatusLines', () => {
		it('should filter by name', async () => {
			const sls = [
				createMockStatusLine({ id: 1, name: 'powerline-pro' }),
				createMockStatusLine({ id: 2, name: 'minimal-bar' }),
				createMockStatusLine({ id: 3, name: 'simple-status' })
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			statuslineLibrary.setSearch('power');

			expect(statuslineLibrary.filteredStatusLines).toHaveLength(1);
			expect(statuslineLibrary.filteredStatusLines[0].name).toBe('powerline-pro');
		});

		it('should filter by description', async () => {
			const sls = [
				createMockStatusLine({ id: 1, name: 'sl-1', description: 'A fancy statusline' }),
				createMockStatusLine({ id: 2, name: 'sl-2', description: 'Basic output' })
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			statuslineLibrary.setSearch('fancy');

			expect(statuslineLibrary.filteredStatusLines).toHaveLength(1);
		});

		it('should filter by tags', async () => {
			const sls = [
				createMockStatusLine({ id: 1, name: 'sl-1', tags: ['minimal', 'fast'] }),
				createMockStatusLine({ id: 2, name: 'sl-2', tags: ['fancy', 'colored'] })
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			statuslineLibrary.setSearch('minimal');

			expect(statuslineLibrary.filteredStatusLines).toHaveLength(1);
			expect(statuslineLibrary.filteredStatusLines[0].id).toBe(1);
		});

		it('should sort with active first, then alphabetical', async () => {
			const sls = [
				createMockStatusLine({ id: 1, name: 'charlie', isActive: false }),
				createMockStatusLine({ id: 2, name: 'alpha', isActive: false }),
				createMockStatusLine({ id: 3, name: 'bravo', isActive: true })
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();

			expect(statuslineLibrary.filteredStatusLines[0].name).toBe('bravo'); // active
			expect(statuslineLibrary.filteredStatusLines[1].name).toBe('alpha');
			expect(statuslineLibrary.filteredStatusLines[2].name).toBe('charlie');
		});
	});

	describe('create', () => {
		it('should create statusline and add to local state', async () => {
			const newSl = createMockStatusLine({ id: 10, name: 'new-sl' });
			vi.mocked(invoke).mockResolvedValueOnce(newSl);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			const result = await statuslineLibrary.create({
				name: 'new-sl',
				statuslineType: 'custom'
			});

			expect(result.id).toBe(10);
			expect(statuslineLibrary.statuslines).toHaveLength(1);
		});
	});

	describe('update', () => {
		it('should update statusline in local state', async () => {
			const sls = [createMockStatusLine({ id: 1, name: 'old-name' })];
			const updated = createMockStatusLine({ id: 1, name: 'new-name' });

			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null) // active
				.mockResolvedValueOnce(updated);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			await statuslineLibrary.update(1, {
				name: 'new-name',
				statuslineType: 'custom'
			});

			expect(statuslineLibrary.statuslines[0].name).toBe('new-name');
		});

		it('should update activeStatusLine if updated statusline is active', async () => {
			const active = createMockStatusLine({ id: 1, name: 'active-sl', isActive: true });
			const sls = [active];
			const updated = createMockStatusLine({ id: 1, name: 'updated-active', isActive: true });

			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(active)
				.mockResolvedValueOnce(updated);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			await statuslineLibrary.update(1, {
				name: 'updated-active',
				statuslineType: 'custom'
			});

			expect(statuslineLibrary.activeStatusLine?.name).toBe('updated-active');
		});
	});

	describe('delete', () => {
		it('should remove statusline from local state', async () => {
			const sls = [
				createMockStatusLine({ id: 1 }),
				createMockStatusLine({ id: 2 })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null)
				.mockResolvedValueOnce(undefined);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			await statuslineLibrary.delete(1);

			expect(statuslineLibrary.statuslines).toHaveLength(1);
			expect(statuslineLibrary.statuslines[0].id).toBe(2);
		});

		it('should clear activeStatusLine if deleted was active', async () => {
			const active = createMockStatusLine({ id: 1, isActive: true });

			vi.mocked(invoke)
				.mockResolvedValueOnce([active])
				.mockResolvedValueOnce(active)
				.mockResolvedValueOnce(undefined);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			await statuslineLibrary.delete(1);

			expect(statuslineLibrary.activeStatusLine).toBeNull();
		});
	});

	describe('activate', () => {
		it('should activate statusline and update local state', async () => {
			const sls = [
				createMockStatusLine({ id: 1, isActive: false }),
				createMockStatusLine({ id: 2, isActive: false })
			];
			const activated = createMockStatusLine({ id: 1, isActive: true });

			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(null)
				.mockResolvedValueOnce(activated);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			await statuslineLibrary.activate(1);

			expect(statuslineLibrary.statuslines[0].isActive).toBe(true);
			expect(statuslineLibrary.statuslines[1].isActive).toBe(false);
			expect(statuslineLibrary.activeStatusLine?.id).toBe(1);
		});
	});

	describe('deactivate', () => {
		it('should deactivate all statuslines', async () => {
			const sls = [
				createMockStatusLine({ id: 1, isActive: true }),
				createMockStatusLine({ id: 2, isActive: false })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(sls)
				.mockResolvedValueOnce(sls[0])
				.mockResolvedValueOnce(undefined);

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.load();
			await statuslineLibrary.deactivate();

			expect(statuslineLibrary.statuslines.every((s) => !s.isActive)).toBe(true);
			expect(statuslineLibrary.activeStatusLine).toBeNull();
		});
	});

	describe('loadGallery', () => {
		it('should load gallery entries from cache', async () => {
			const entries = [
				createMockGalleryEntry({ name: 'entry-1' }),
				createMockGalleryEntry({ name: 'entry-2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(entries) // get_statusline_gallery_cache
				.mockResolvedValueOnce(entries); // fetch_statusline_gallery (background)

			const { statuslineLibrary } = await import('$lib/stores/statuslineLibrary.svelte');
			await statuslineLibrary.loadGallery();

			expect(statuslineLibrary.gallery).toHaveLength(2);
			expect(statuslineLibrary.isGalleryLoading).toBe(false);
		});
	});
});
