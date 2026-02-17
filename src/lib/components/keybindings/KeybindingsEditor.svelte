<script lang="ts">
	import { keybindingsLibrary, notifications } from '$lib/stores';
	import {
		KEYBINDING_CONTEXTS,
		formatKeystroke
	} from '$lib/types';
	import type { KeybindingContext, MergedBinding } from '$lib/types';
	import KeyCaptureDialog from './KeyCaptureDialog.svelte';
	import {
		ChevronDown,
		ChevronRight,
		Search,
		RotateCcw,
		Save,
		Pencil,
		X,
		Plus,
		Undo2
	} from 'lucide-svelte';

	// Key capture dialog state
	let captureContext = $state<KeybindingContext | null>(null);
	let captureAction = $state('');
	let captureActionLabel = $state('');
	let captureCurrentKeys = $state<string[]>([]);

	function openCapture(binding: MergedBinding) {
		captureContext = binding.context;
		captureAction = binding.action;
		captureActionLabel = binding.label;
		captureCurrentKeys = binding.currentKeys;
	}

	function handleCaptureConfirm(key: string) {
		if (captureContext && captureAction) {
			keybindingsLibrary.setBinding(captureContext, key, captureAction);
		}
		closeCapture();
	}

	function closeCapture() {
		captureContext = null;
		captureAction = '';
		captureActionLabel = '';
		captureCurrentKeys = [];
	}

	function unbindKey(context: KeybindingContext, key: string) {
		keybindingsLibrary.unbindKey(context, key);
	}

	function removeKeyFromAction(context: KeybindingContext, key: string, binding: MergedBinding) {
		// If it's an added key (override), remove the override
		if (binding.addedKeys.includes(key)) {
			keybindingsLibrary.removeOverride(context, key);
		} else {
			// It's a default key — unbind it
			keybindingsLibrary.unbindKey(context, key);
		}
	}

	function resetBinding(binding: MergedBinding) {
		// Remove all overrides for this binding's keys
		const overrideBlock = keybindingsLibrary.overrides.find((b) => b.context === binding.context);
		if (overrideBlock) {
			// Remove added keys
			for (const key of binding.addedKeys) {
				keybindingsLibrary.removeOverride(binding.context, key);
			}
			// Remove unbind overrides (null values for default keys)
			for (const key of binding.unboundKeys) {
				keybindingsLibrary.removeOverride(binding.context, key);
			}
		}
	}

	function resetContext(context: KeybindingContext) {
		keybindingsLibrary.resetContext(context);
	}

	function resetAll() {
		keybindingsLibrary.resetAll();
	}

	async function handleSave() {
		try {
			await keybindingsLibrary.save();
			notifications.success('Keybindings saved to ~/.claude/keybindings.json');
		} catch {
			notifications.error('Failed to save keybindings');
		}
	}

	function contextHasOverrides(context: string): boolean {
		const block = keybindingsLibrary.overrides.find((b) => b.context === context);
		return block ? Object.keys(block.bindings).length > 0 : false;
	}

	function getContextOverrideCount(context: string): number {
		const block = keybindingsLibrary.overrides.find((b) => b.context === context);
		return block ? Object.keys(block.bindings).length : 0;
	}
</script>

<div class="space-y-4">
	<!-- Toolbar -->
	<div class="flex flex-wrap items-center gap-3">
		<!-- Search -->
		<div class="relative flex-1 min-w-[200px]">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
			<input
				type="text"
				placeholder="Search actions, keys..."
				bind:value={keybindingsLibrary.searchQuery}
				class="input text-sm pl-9 w-full"
			/>
		</div>

		<!-- Actions -->
		<div class="flex items-center gap-2">
			{#if keybindingsLibrary.overrideCount > 0}
				<span class="px-2 py-1 text-xs font-medium rounded-full bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300">
					{keybindingsLibrary.overrideCount} override{keybindingsLibrary.overrideCount !== 1 ? 's' : ''}
				</span>
			{/if}

			<button
				onclick={() => keybindingsLibrary.expandAll()}
				class="btn btn-ghost text-xs"
				title="Expand all"
			>
				Expand All
			</button>
			<button
				onclick={() => keybindingsLibrary.collapseAll()}
				class="btn btn-ghost text-xs"
				title="Collapse all"
			>
				Collapse All
			</button>

			{#if keybindingsLibrary.overrideCount > 0}
				<button
					onclick={resetAll}
					class="btn btn-ghost text-xs text-red-500 hover:text-red-700"
					title="Reset all overrides"
				>
					<RotateCcw class="w-3.5 h-3.5 mr-1" />
					Reset All
				</button>
			{/if}
		</div>
	</div>

	<!-- Context Sections -->
	{#each KEYBINDING_CONTEXTS as ctxInfo}
		{@const bindings = keybindingsLibrary.filteredByContext.get(ctxInfo.context)}
		{#if bindings && bindings.length > 0}
			{@const isExpanded = keybindingsLibrary.expandedContexts.has(ctxInfo.context)}
			{@const overrideCount = getContextOverrideCount(ctxInfo.context)}
			<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
				<!-- Context Header -->
				<button
					onclick={() => keybindingsLibrary.toggleContext(ctxInfo.context)}
					class="w-full flex items-center justify-between px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
				>
					<div class="flex items-center gap-2">
						{#if isExpanded}
							<ChevronDown class="w-4 h-4 text-gray-400" />
						{:else}
							<ChevronRight class="w-4 h-4 text-gray-400" />
						{/if}
						<span class="font-medium text-gray-900 dark:text-white">{ctxInfo.label}</span>
						<span class="text-xs text-gray-400 dark:text-gray-500">{ctxInfo.description}</span>
						{#if overrideCount > 0}
							<span class="px-1.5 py-0.5 text-xs rounded-full bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300">
								{overrideCount}
							</span>
						{/if}
					</div>
					{#if contextHasOverrides(ctxInfo.context)}
						<button
							onclick={(e) => { e.stopPropagation(); resetContext(ctxInfo.context); }}
							class="text-xs text-red-500 hover:text-red-700 px-2 py-1 rounded hover:bg-red-50 dark:hover:bg-red-900/20"
							title="Reset context overrides"
						>
							Reset
						</button>
					{/if}
				</button>

				<!-- Bindings List -->
				{#if isExpanded}
					<div class="border-t border-gray-200 dark:border-gray-700 divide-y divide-gray-100 dark:divide-gray-700/50">
						{#each bindings as binding}
							{@const hasModification = binding.isModified}
							<div
								class="flex items-center gap-3 px-4 py-2.5 text-sm
									{hasModification ? 'bg-primary-50/50 dark:bg-primary-900/10' : ''}"
							>
								<!-- Action info -->
								<div class="flex-1 min-w-0">
									<div class="flex items-center gap-2">
										<span class="font-medium text-gray-900 dark:text-white truncate">
											{binding.label}
										</span>
										{#if hasModification}
											<span class="w-1.5 h-1.5 rounded-full bg-primary-500 flex-shrink-0" title="Modified"></span>
										{/if}
									</div>
									<div class="text-xs text-gray-400 dark:text-gray-500 truncate">
										{binding.description}
									</div>
								</div>

								<!-- Default keys (gray) -->
								{#if binding.defaultKeys.length > 0}
									<div class="flex items-center gap-1 flex-shrink-0">
										{#each binding.defaultKeys as key}
											{@const isUnbound = binding.unboundKeys.includes(key)}
											<span
												class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-mono
													{isUnbound
													? 'bg-gray-100 dark:bg-gray-700 text-gray-400 dark:text-gray-500 line-through'
													: 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300'}"
												title={isUnbound ? 'Unbound (default was ' + formatKeystroke(key) + ')' : 'Default: ' + formatKeystroke(key)}
											>
												{formatKeystroke(key)}
											</span>
										{/each}
									</div>
								{/if}

								<!-- Current keys (colored, active) -->
								<div class="flex items-center gap-1 flex-shrink-0">
									{#each binding.addedKeys as key}
										<span
											class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-mono bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300"
											title="Custom binding"
										>
											{formatKeystroke(key)}
											<button
												onclick={() => removeKeyFromAction(binding.context, key, binding)}
												class="hover:text-red-500"
												title="Remove this binding"
											>
												<X class="w-3 h-3" />
											</button>
										</span>
									{/each}
								</div>

								<!-- Actions -->
								<div class="flex items-center gap-1 flex-shrink-0">
									<button
										onclick={() => openCapture(binding)}
										class="p-1 rounded text-gray-400 hover:text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/20"
										title="Add keybinding"
									>
										<Plus class="w-4 h-4" />
									</button>
									{#if hasModification}
										<button
											onclick={() => resetBinding(binding)}
											class="p-1 rounded text-gray-400 hover:text-yellow-600 hover:bg-yellow-50 dark:hover:bg-yellow-900/20"
											title="Reset to default"
										>
											<Undo2 class="w-4 h-4" />
										</button>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	{/each}

	<!-- Save Button -->
	<div class="flex justify-end pt-2">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Keybindings
		</button>
	</div>
</div>

<!-- Key Capture Dialog -->
{#if captureContext}
	<KeyCaptureDialog
		context={captureContext}
		action={captureAction}
		actionLabel={captureActionLabel}
		currentKeys={captureCurrentKeys}
		onconfirm={handleCaptureConfirm}
		oncancel={closeCapture}
	/>
{/if}
