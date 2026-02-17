import { invoke } from '@tauri-apps/api/core';
import type {
	ContextBindings,
	KeybindingsFile,
	KeybindingContext,
	MergedBinding,
	KeyConflict
} from '$lib/types';
import {
	KEYBINDING_ACTIONS,
	KEYBINDING_CONTEXTS,
	RESERVED_KEYS,
	TERMINAL_CONFLICT_KEYS,
	getActionsForContext
} from '$lib/types';

class KeybindingsLibraryState {
	overrides = $state<ContextBindings[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	expandedContexts = $state<Set<string>>(new Set());

	/** Merged bindings grouped by context */
	mergedByContext = $derived.by(() => this.computeMerged());

	/** Total number of user overrides */
	overrideCount = $derived.by(() => {
		let count = 0;
		for (const block of this.overrides) {
			count += Object.keys(block.bindings).length;
		}
		return count;
	});

	/** Filtered merged bindings (respects searchQuery) */
	filteredByContext = $derived.by(() => {
		const q = this.searchQuery.toLowerCase().trim();
		if (!q) return this.mergedByContext;

		const filtered = new Map<KeybindingContext, MergedBinding[]>();
		for (const [ctx, bindings] of this.mergedByContext) {
			const matched = bindings.filter(
				(b) =>
					b.label.toLowerCase().includes(q) ||
					b.action.toLowerCase().includes(q) ||
					b.description.toLowerCase().includes(q) ||
					b.currentKeys.some((k) => k.toLowerCase().includes(q))
			);
			if (matched.length > 0) {
				filtered.set(ctx, matched);
			}
		}
		return filtered;
	});

	// ========================================================================
	// Load / Save
	// ========================================================================

	async load() {
		console.log('[keybindingsLibrary] Loading keybindings...');
		this.isLoading = true;
		this.error = null;
		try {
			const file = await invoke<KeybindingsFile>('get_keybindings');
			this.overrides = file.bindings ?? [];
			console.log(
				`[keybindingsLibrary] Loaded ${this.overrides.length} context overrides`
			);
		} catch (e) {
			this.error = String(e);
			console.error('[keybindingsLibrary] Failed to load keybindings:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async save(): Promise<void> {
		console.log('[keybindingsLibrary] Saving keybindings...');
		try {
			const file: KeybindingsFile = {
				schema: 'https://www.schemastore.org/claude-code-keybindings.json',
				bindings: this.overrides.filter((b) => Object.keys(b.bindings).length > 0)
			};
			await invoke('save_keybindings', { keybindings: file });
			console.log('[keybindingsLibrary] Saved successfully');
		} catch (e) {
			this.error = String(e);
			console.error('[keybindingsLibrary] Failed to save:', e);
			throw e;
		}
	}

	// ========================================================================
	// Mutation Methods
	// ========================================================================

	/** Add or update a binding override: key → action */
	setBinding(context: KeybindingContext, key: string, action: string) {
		const block = this.getOrCreateBlock(context);
		block.bindings[key] = action;
		this.overrides = [...this.overrides];
	}

	/** Unbind a key (set override to null) */
	unbindKey(context: KeybindingContext, key: string) {
		const block = this.getOrCreateBlock(context);
		block.bindings[key] = null;
		this.overrides = [...this.overrides];
	}

	/** Remove a user override for a specific key (revert to default) */
	removeOverride(context: KeybindingContext, key: string) {
		const block = this.findBlock(context);
		if (block) {
			delete block.bindings[key];
			this.overrides = [...this.overrides];
		}
	}

	/** Reset all overrides for a context */
	resetContext(context: KeybindingContext) {
		this.overrides = this.overrides.filter((b) => b.context !== context);
	}

	/** Reset all overrides */
	resetAll() {
		this.overrides = [];
	}

	/** Toggle expanded state for a context */
	toggleContext(context: string) {
		const next = new Set(this.expandedContexts);
		if (next.has(context)) {
			next.delete(context);
		} else {
			next.add(context);
		}
		this.expandedContexts = next;
	}

	/** Expand all contexts */
	expandAll() {
		this.expandedContexts = new Set(KEYBINDING_CONTEXTS.map((c) => c.context));
	}

	/** Collapse all contexts */
	collapseAll() {
		this.expandedContexts = new Set();
	}

	// ========================================================================
	// Conflict Detection
	// ========================================================================

	/** Detect conflicts for a key in a given context */
	detectConflicts(
		context: KeybindingContext,
		key: string,
		excludeAction?: string
	): KeyConflict[] {
		const conflicts: KeyConflict[] = [];
		const contextSet = new Set<KeybindingContext>([context]);

		// Global context conflicts with everything
		if (context !== 'Global') {
			contextSet.add('Global');
		}

		for (const checkCtx of contextSet) {
			const merged = this.mergedByContext.get(checkCtx) ?? [];
			for (const binding of merged) {
				if (excludeAction && binding.action === excludeAction) continue;
				if (binding.currentKeys.includes(key)) {
					conflicts.push({
						key,
						context: checkCtx,
						existingAction: binding.action,
						existingActionLabel: binding.label
					});
				}
			}
		}

		// Also check if the key is used in the target context when checking Global
		if (context === 'Global') {
			for (const [ctx, bindings] of this.mergedByContext) {
				if (ctx === 'Global') continue;
				for (const binding of bindings) {
					if (excludeAction && binding.action === excludeAction) continue;
					if (binding.currentKeys.includes(key)) {
						conflicts.push({
							key,
							context: ctx,
							existingAction: binding.action,
							existingActionLabel: binding.label
						});
					}
				}
			}
		}

		return conflicts;
	}

	/** Check if a key is reserved and cannot be rebound */
	isReservedKey(key: string): boolean {
		return RESERVED_KEYS.includes(key.toLowerCase());
	}

	/** Check if a key has a terminal conflict warning */
	isTerminalConflict(key: string): boolean {
		return TERMINAL_CONFLICT_KEYS.includes(key.toLowerCase());
	}

	// ========================================================================
	// Merge Algorithm
	// ========================================================================

	private computeMerged(): Map<KeybindingContext, MergedBinding[]> {
		const result = new Map<KeybindingContext, MergedBinding[]>();

		for (const ctxInfo of KEYBINDING_CONTEXTS) {
			const ctx = ctxInfo.context;
			const actions = getActionsForContext(ctx);
			const overrideBlock = this.findBlock(ctx);
			const overrides = overrideBlock?.bindings ?? {};

			// Build a map of action → keys based on defaults
			const actionKeyMap = new Map<string, Set<string>>();
			for (const action of actions) {
				actionKeyMap.set(action.action, new Set(action.defaultKeys));
			}

			// Track which defaults were unbound and which keys were added
			const unboundMap = new Map<string, string[]>();
			const addedMap = new Map<string, string[]>();

			// Apply overrides
			for (const [key, actionOrNull] of Object.entries(overrides)) {
				if (actionOrNull === null) {
					// Unbind: remove this key from whichever action has it as default
					for (const action of actions) {
						const keySet = actionKeyMap.get(action.action);
						if (keySet && action.defaultKeys.includes(key)) {
							keySet.delete(key);
							const existing = unboundMap.get(action.action) ?? [];
							existing.push(key);
							unboundMap.set(action.action, existing);
						}
					}
				} else {
					// Rebind: first remove this key from any action that has it by default
					for (const action of actions) {
						const keySet = actionKeyMap.get(action.action);
						if (keySet && keySet.has(key) && action.action !== actionOrNull) {
							keySet.delete(key);
						}
					}

					// Then add the key to the target action
					if (!actionKeyMap.has(actionOrNull)) {
						// Could be a custom command:* binding
						actionKeyMap.set(actionOrNull, new Set());
					}
					actionKeyMap.get(actionOrNull)!.add(key);

					// Track as added if not a default for this action
					const targetAction = actions.find((a) => a.action === actionOrNull);
					if (!targetAction || !targetAction.defaultKeys.includes(key)) {
						const existing = addedMap.get(actionOrNull) ?? [];
						existing.push(key);
						addedMap.set(actionOrNull, existing);
					}
				}
			}

			// Build merged bindings
			const merged: MergedBinding[] = [];
			const processedActions = new Set<string>();

			for (const action of actions) {
				const currentKeys = Array.from(actionKeyMap.get(action.action) ?? []);
				const unboundKeys = unboundMap.get(action.action) ?? [];
				const addedKeys = addedMap.get(action.action) ?? [];
				const isModified =
					unboundKeys.length > 0 ||
					addedKeys.length > 0 ||
					currentKeys.sort().join(',') !== [...action.defaultKeys].sort().join(',');

				merged.push({
					action: action.action,
					label: action.label,
					description: action.description,
					context: ctx,
					defaultKeys: action.defaultKeys,
					currentKeys,
					isModified,
					unboundKeys,
					addedKeys
				});
				processedActions.add(action.action);
			}

			// Add any custom command:* overrides not in the known actions
			for (const [key, actionOrNull] of Object.entries(overrides)) {
				if (actionOrNull && !processedActions.has(actionOrNull)) {
					const existingMerged = merged.find((m) => m.action === actionOrNull);
					if (!existingMerged) {
						merged.push({
							action: actionOrNull,
							label: actionOrNull.startsWith('command:')
								? actionOrNull.slice('command:'.length)
								: actionOrNull,
							description: 'Custom binding',
							context: ctx,
							defaultKeys: [],
							currentKeys: Array.from(actionKeyMap.get(actionOrNull) ?? []),
							isModified: true,
							unboundKeys: [],
							addedKeys: [key]
						});
						processedActions.add(actionOrNull);
					}
				}
			}

			result.set(ctx, merged);
		}

		return result;
	}

	// ========================================================================
	// Private Helpers
	// ========================================================================

	private findBlock(context: KeybindingContext): ContextBindings | undefined {
		return this.overrides.find((b) => b.context === context);
	}

	private getOrCreateBlock(context: KeybindingContext): ContextBindings {
		let block = this.findBlock(context);
		if (!block) {
			block = { context, bindings: {} };
			this.overrides = [...this.overrides, block];
		}
		return block;
	}
}

export const keybindingsLibrary = new KeybindingsLibraryState();
