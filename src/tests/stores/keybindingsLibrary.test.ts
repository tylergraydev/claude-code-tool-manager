import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

vi.mock('$lib/types', () => {
	const contexts = [
		{ context: 'Global', label: 'Global' },
		{ context: 'Input', label: 'Input' }
	];
	const actions = [
		{ action: 'submit', label: 'Submit', description: 'Submit message', defaultKeys: ['Enter'], context: 'Global' },
		{ action: 'cancel', label: 'Cancel', description: 'Cancel operation', defaultKeys: ['Escape'], context: 'Global' },
		{ action: 'newline', label: 'New Line', description: 'Insert newline', defaultKeys: ['Shift+Enter'], context: 'Input' }
	];
	return {
		KEYBINDING_CONTEXTS: contexts,
		KEYBINDING_ACTIONS: actions,
		RESERVED_KEYS: ['ctrl+c', 'ctrl+z'],
		TERMINAL_CONFLICT_KEYS: ['ctrl+w'],
		getActionsForContext: (ctx: string) => actions.filter((a) => a.context === ctx)
	};
});

describe('Keybindings Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load keybindings successfully', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ bindings: [{ context: 'Global', bindings: { 'Ctrl+S': 'submit' } }] });
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			await keybindingsLibrary.load();
			expect(keybindingsLibrary.overrides).toHaveLength(1);
			expect(keybindingsLibrary.isLoading).toBe(false);
		});

		it('should handle null bindings', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ bindings: null });
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			await keybindingsLibrary.load();
			expect(keybindingsLibrary.overrides).toEqual([]);
		});

		it('should handle load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			await keybindingsLibrary.load();
			expect(keybindingsLibrary.error).toBe('Error: fail');
		});
	});

	describe('save', () => {
		it('should save keybindings', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+S', 'submit');
			await keybindingsLibrary.save();
			expect(invoke).toHaveBeenCalledWith('save_keybindings', expect.any(Object));
		});

		it('should filter out empty context blocks on save', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			// Add and then remove an override
			keybindingsLibrary.setBinding('Global', 'Ctrl+S', 'submit');
			keybindingsLibrary.removeOverride('Global', 'Ctrl+S');
			await keybindingsLibrary.save();
			const call = vi.mocked(invoke).mock.calls[0];
			const file = (call[1] as any).keybindings;
			expect(file.bindings).toEqual([]);
		});

		it('should throw on save error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			await expect(keybindingsLibrary.save()).rejects.toThrow('fail');
			expect(keybindingsLibrary.error).toBe('Error: fail');
		});
	});

	describe('setBinding', () => {
		it('should add binding to existing context', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+K', 'cancel');
			const block = keybindingsLibrary.overrides.find((b) => b.context === 'Global');
			expect(block?.bindings['Ctrl+K']).toBe('cancel');
		});

		it('should create new context block if needed', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Input', 'Ctrl+A', 'newline');
			const block = keybindingsLibrary.overrides.find((b) => b.context === 'Input');
			expect(block?.bindings['Ctrl+A']).toBe('newline');
		});
	});

	describe('unbindKey', () => {
		it('should set key to null', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.unbindKey('Global', 'Enter');
			const block = keybindingsLibrary.overrides.find((b) => b.context === 'Global');
			expect(block?.bindings['Enter']).toBeNull();
		});
	});

	describe('removeOverride', () => {
		it('should remove a key override', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+K', 'cancel');
			keybindingsLibrary.removeOverride('Global', 'Ctrl+K');
			const block = keybindingsLibrary.overrides.find((b) => b.context === 'Global');
			expect(block?.bindings['Ctrl+K']).toBeUndefined();
		});

		it('should handle removing from non-existent context', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.removeOverride('Global', 'Ctrl+K');
			// Should not throw
		});
	});

	describe('resetContext', () => {
		it('should remove all overrides for a context', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+K', 'cancel');
			keybindingsLibrary.setBinding('Global', 'Ctrl+S', 'submit');
			keybindingsLibrary.resetContext('Global');
			expect(keybindingsLibrary.overrides.find((b) => b.context === 'Global')).toBeUndefined();
		});
	});

	describe('resetAll', () => {
		it('should clear all overrides', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+K', 'cancel');
			keybindingsLibrary.setBinding('Input', 'Ctrl+A', 'newline');
			keybindingsLibrary.resetAll();
			expect(keybindingsLibrary.overrides).toEqual([]);
		});
	});

	describe('overrideCount', () => {
		it('should count total overrides', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+K', 'cancel');
			keybindingsLibrary.setBinding('Global', 'Ctrl+S', 'submit');
			keybindingsLibrary.setBinding('Input', 'Ctrl+A', 'newline');
			expect(keybindingsLibrary.overrideCount).toBe(3);
		});
	});

	describe('context expansion', () => {
		it('should toggle context expansion', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.toggleContext('Global');
			expect(keybindingsLibrary.expandedContexts.has('Global')).toBe(true);
			keybindingsLibrary.toggleContext('Global');
			expect(keybindingsLibrary.expandedContexts.has('Global')).toBe(false);
		});

		it('should expand all', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.expandAll();
			expect(keybindingsLibrary.expandedContexts.has('Global')).toBe(true);
			expect(keybindingsLibrary.expandedContexts.has('Input')).toBe(true);
		});

		it('should collapse all', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.expandAll();
			keybindingsLibrary.collapseAll();
			expect(keybindingsLibrary.expandedContexts.size).toBe(0);
		});
	});

	describe('conflict detection', () => {
		it('should detect conflict in same context', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			// 'Enter' is the default key for 'submit' in Global
			const conflicts = keybindingsLibrary.detectConflicts('Global', 'Enter');
			expect(conflicts.length).toBeGreaterThan(0);
			expect(conflicts[0].existingAction).toBe('submit');
		});

		it('should exclude current action from conflicts', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			const conflicts = keybindingsLibrary.detectConflicts('Global', 'Enter', 'submit');
			expect(conflicts.find((c) => c.existingAction === 'submit')).toBeUndefined();
		});

		it('should check Global context for non-global contexts', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			const conflicts = keybindingsLibrary.detectConflicts('Input', 'Enter');
			// Should find conflict with Global's 'submit' action on Enter
			const globalConflict = conflicts.find((c) => c.context === 'Global');
			expect(globalConflict).toBeDefined();
		});

		it('should check all contexts when context is Global', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			const conflicts = keybindingsLibrary.detectConflicts('Global', 'Shift+Enter');
			// Shift+Enter is default for 'newline' in Input context
			const inputConflict = conflicts.find((c) => c.context === 'Input');
			expect(inputConflict).toBeDefined();
		});
	});

	describe('reserved and terminal keys', () => {
		it('should identify reserved keys', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			expect(keybindingsLibrary.isReservedKey('Ctrl+C')).toBe(true);
			expect(keybindingsLibrary.isReservedKey('Ctrl+Z')).toBe(true);
			expect(keybindingsLibrary.isReservedKey('Ctrl+K')).toBe(false);
		});

		it('should identify terminal conflict keys', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			expect(keybindingsLibrary.isTerminalConflict('Ctrl+W')).toBe(true);
			expect(keybindingsLibrary.isTerminalConflict('Ctrl+K')).toBe(false);
		});
	});

	describe('mergedByContext', () => {
		it('should merge default and override bindings', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+S', 'submit');

			const globalMerged = keybindingsLibrary.mergedByContext.get('Global');
			expect(globalMerged).toBeDefined();

			const submitBinding = globalMerged?.find((b) => b.action === 'submit');
			expect(submitBinding).toBeDefined();
			expect(submitBinding?.currentKeys).toContain('Ctrl+S');
			expect(submitBinding?.isModified).toBe(true);
		});

		it('should handle unbind overrides', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.unbindKey('Global', 'Enter');

			const globalMerged = keybindingsLibrary.mergedByContext.get('Global');
			const submitBinding = globalMerged?.find((b) => b.action === 'submit');
			expect(submitBinding?.currentKeys).not.toContain('Enter');
			expect(submitBinding?.unboundKeys).toContain('Enter');
			expect(submitBinding?.isModified).toBe(true);
		});

		it('should handle custom command bindings', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Ctrl+M', 'command:my-cmd');

			const globalMerged = keybindingsLibrary.mergedByContext.get('Global');
			const customBinding = globalMerged?.find((b) => b.action === 'command:my-cmd');
			expect(customBinding).toBeDefined();
			expect(customBinding?.label).toBe('my-cmd');
			expect(customBinding?.description).toBe('Custom binding');
			expect(customBinding?.currentKeys).toContain('Ctrl+M');
		});

		it('should rebind key from one action to another', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.setBinding('Global', 'Enter', 'cancel');

			const globalMerged = keybindingsLibrary.mergedByContext.get('Global');
			const cancelBinding = globalMerged?.find((b) => b.action === 'cancel');
			expect(cancelBinding?.currentKeys).toContain('Enter');
			const submitBinding = globalMerged?.find((b) => b.action === 'submit');
			expect(submitBinding?.currentKeys).not.toContain('Enter');
		});
	});

	describe('filteredByContext', () => {
		it('should return all when no search', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			expect(keybindingsLibrary.filteredByContext).toBe(keybindingsLibrary.mergedByContext);
		});

		it('should filter by search query', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.searchQuery = 'submit';
			const filtered = keybindingsLibrary.filteredByContext;
			const globalBindings = filtered.get('Global');
			expect(globalBindings?.every((b) =>
				b.label.toLowerCase().includes('submit') ||
				b.action.toLowerCase().includes('submit') ||
				b.description.toLowerCase().includes('submit') ||
				b.currentKeys.some((k) => k.toLowerCase().includes('submit'))
			)).toBe(true);
		});

		it('should filter by key name', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.searchQuery = 'enter';
			const filtered = keybindingsLibrary.filteredByContext;
			const globalBindings = filtered.get('Global') ?? [];
			expect(globalBindings.length).toBeGreaterThan(0);
		});

		it('should exclude contexts with no matches', async () => {
			const { keybindingsLibrary } = await import('$lib/stores/keybindingsLibrary.svelte');
			keybindingsLibrary.searchQuery = 'newline';
			const filtered = keybindingsLibrary.filteredByContext;
			expect(filtered.has('Global')).toBe(false);
			expect(filtered.has('Input')).toBe(true);
		});
	});
});
