import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Spinner Verb Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load verbs and mode', async () => {
			const mockVerbs = [
				{ id: 1, verb: 'Thinking', isEnabled: true, displayOrder: 0 },
				{ id: 2, verb: 'Coding', isEnabled: true, displayOrder: 1 }
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(mockVerbs) // get_all_spinner_verbs
				.mockResolvedValueOnce('replace'); // get_spinner_verb_mode

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.load();

			expect(spinnerVerbLibrary.verbs).toHaveLength(2);
			expect(spinnerVerbLibrary.verbs[0].verb).toBe('Thinking');
			expect(spinnerVerbLibrary.mode).toBe('replace');
			expect(spinnerVerbLibrary.isLoading).toBe(false);
			expect(spinnerVerbLibrary.error).toBeNull();
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			const loadPromise = spinnerVerbLibrary.load();

			expect(spinnerVerbLibrary.isLoading).toBe(true);

			vi.mocked(invoke).mockResolvedValueOnce('append');
			resolveInvoke!([]);
			await loadPromise;

			expect(spinnerVerbLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('DB error'));

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.load();

			expect(spinnerVerbLibrary.error).toContain('DB error');
			expect(spinnerVerbLibrary.isLoading).toBe(false);
		});
	});

	describe('default state', () => {
		it('should have correct defaults', async () => {
			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			expect(spinnerVerbLibrary.verbs).toEqual([]);
			expect(spinnerVerbLibrary.mode).toBe('append');
			expect(spinnerVerbLibrary.isLoading).toBe(false);
			expect(spinnerVerbLibrary.error).toBeNull();
		});
	});

	describe('create', () => {
		it('should create a verb and add to list', async () => {
			const newVerb = { id: 1, verb: 'Pondering', isEnabled: true, displayOrder: 0 };
			vi.mocked(invoke).mockResolvedValueOnce(newVerb);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			const result = await spinnerVerbLibrary.create('Pondering');

			expect(result.id).toBe(1);
			expect(result.verb).toBe('Pondering');
			expect(spinnerVerbLibrary.verbs).toHaveLength(1);
			expect(invoke).toHaveBeenCalledWith('create_spinner_verb', { verb: 'Pondering' });
		});
	});

	describe('update', () => {
		it('should update a verb in the list', async () => {
			const mockVerbs = [
				{ id: 1, verb: 'Old', isEnabled: true, displayOrder: 0 }
			];
			const updatedVerb = { id: 1, verb: 'New', isEnabled: false, displayOrder: 0 };

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockVerbs)
				.mockResolvedValueOnce('append')
				.mockResolvedValueOnce(updatedVerb);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.load();

			const result = await spinnerVerbLibrary.update(1, 'New', false);

			expect(result.verb).toBe('New');
			expect(spinnerVerbLibrary.verbs[0].verb).toBe('New');
			expect(spinnerVerbLibrary.verbs[0].isEnabled).toBe(false);
			expect(invoke).toHaveBeenCalledWith('update_spinner_verb', { id: 1, verb: 'New', isEnabled: false });
		});
	});

	describe('delete', () => {
		it('should delete a verb from the list', async () => {
			const mockVerbs = [
				{ id: 1, verb: 'Keep', isEnabled: true, displayOrder: 0 },
				{ id: 2, verb: 'Remove', isEnabled: true, displayOrder: 1 }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockVerbs)
				.mockResolvedValueOnce('append')
				.mockResolvedValueOnce(undefined);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.load();
			await spinnerVerbLibrary.delete(2);

			expect(spinnerVerbLibrary.verbs).toHaveLength(1);
			expect(spinnerVerbLibrary.verbs[0].id).toBe(1);
			expect(invoke).toHaveBeenCalledWith('delete_spinner_verb', { id: 2 });
		});
	});

	describe('reorder', () => {
		it('should reorder verbs by given IDs', async () => {
			const mockVerbs = [
				{ id: 1, verb: 'First', isEnabled: true, displayOrder: 0 },
				{ id: 2, verb: 'Second', isEnabled: true, displayOrder: 1 },
				{ id: 3, verb: 'Third', isEnabled: true, displayOrder: 2 }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockVerbs)
				.mockResolvedValueOnce('append')
				.mockResolvedValueOnce(undefined);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.load();
			await spinnerVerbLibrary.reorder([3, 1, 2]);

			expect(spinnerVerbLibrary.verbs[0].id).toBe(3);
			expect(spinnerVerbLibrary.verbs[0].displayOrder).toBe(0);
			expect(spinnerVerbLibrary.verbs[1].id).toBe(1);
			expect(spinnerVerbLibrary.verbs[1].displayOrder).toBe(1);
			expect(spinnerVerbLibrary.verbs[2].id).toBe(2);
			expect(spinnerVerbLibrary.verbs[2].displayOrder).toBe(2);
			expect(invoke).toHaveBeenCalledWith('reorder_spinner_verbs', { ids: [3, 1, 2] });
		});

		it('should filter out verbs with IDs not in the provided list', async () => {
			const mockVerbs = [
				{ id: 1, verb: 'First', isEnabled: true, displayOrder: 0 },
				{ id: 2, verb: 'Second', isEnabled: true, displayOrder: 1 }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockVerbs)
				.mockResolvedValueOnce('append')
				.mockResolvedValueOnce(undefined);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.load();
			// Reorder with an ID that doesn't exist - should filter it out (null case)
			await spinnerVerbLibrary.reorder([999, 1]);

			expect(spinnerVerbLibrary.verbs).toHaveLength(1);
			expect(spinnerVerbLibrary.verbs[0].id).toBe(1);
		});
	});

	describe('setMode', () => {
		it('should set mode to replace', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.setMode('replace');

			expect(spinnerVerbLibrary.mode).toBe('replace');
			expect(invoke).toHaveBeenCalledWith('set_spinner_verb_mode', { mode: 'replace' });
		});

		it('should set mode to append', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.setMode('append');

			expect(spinnerVerbLibrary.mode).toBe('append');
			expect(invoke).toHaveBeenCalledWith('set_spinner_verb_mode', { mode: 'append' });
		});
	});

	describe('sync', () => {
		it('should sync to settings.json', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { spinnerVerbLibrary } = await import('$lib/stores/spinnerVerbLibrary.svelte');
			await spinnerVerbLibrary.sync();

			expect(invoke).toHaveBeenCalledWith('sync_spinner_verbs');
		});
	});
});
