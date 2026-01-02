<script lang="ts">
	import { hookLibrary, notifications } from '$lib/stores';
	import type { Hook } from '$lib/types';
	import { Download, Copy, X, Check, FileJson, CheckSquare, Square } from 'lucide-svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import { writeTextFile } from '@tauri-apps/plugin-fs';

	type Props = {
		onClose: () => void;
	};

	let { onClose }: Props = $props();

	let selectedIds = $state<Set<number>>(new Set());
	let exportPreview = $state('');
	let isExporting = $state(false);
	let copiedToClipboard = $state(false);

	// All non-template hooks
	const availableHooks = $derived(hookLibrary.hooks.filter((h) => !h.isTemplate));

	// Selected hooks
	const selectedHooks = $derived(availableHooks.filter((h) => selectedIds.has(h.id)));

	// Update preview when selection changes
	$effect(() => {
		if (selectedIds.size > 0) {
			updatePreview();
		} else {
			exportPreview = '';
		}
	});

	async function updatePreview() {
		try {
			const ids = Array.from(selectedIds);
			exportPreview = await hookLibrary.exportToJson(ids);
		} catch (e) {
			console.error('Failed to generate preview:', e);
		}
	}

	function toggleHook(id: number) {
		const newSet = new Set(selectedIds);
		if (newSet.has(id)) {
			newSet.delete(id);
		} else {
			newSet.add(id);
		}
		selectedIds = newSet;
	}

	function selectAll() {
		selectedIds = new Set(availableHooks.map((h) => h.id));
	}

	function deselectAll() {
		selectedIds = new Set();
	}

	async function copyToClipboard() {
		if (selectedIds.size === 0) return;

		try {
			await hookLibrary.exportToClipboard(Array.from(selectedIds));
			copiedToClipboard = true;
			notifications.success('Copied to clipboard');
			setTimeout(() => {
				copiedToClipboard = false;
			}, 2000);
		} catch (e) {
			notifications.error('Failed to copy to clipboard');
		}
	}

	async function exportToFile() {
		if (selectedIds.size === 0) return;

		try {
			isExporting = true;

			const filePath = await save({
				defaultPath: 'hooks-export.json',
				filters: [{ name: 'JSON', extensions: ['json'] }]
			});

			if (filePath) {
				const json = await hookLibrary.exportToJson(Array.from(selectedIds));
				await writeTextFile(filePath, json);
				notifications.success(`Exported ${selectedIds.size} hooks`);
				onClose();
			}
		} catch (e) {
			notifications.error('Failed to export hooks');
		} finally {
			isExporting = false;
		}
	}
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
	<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-3xl w-full mx-4 max-h-[85vh] flex flex-col overflow-hidden">
		<!-- Header -->
		<div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/50 flex items-center justify-center">
					<FileJson class="w-5 h-5 text-blue-600 dark:text-blue-400" />
				</div>
				<div>
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Export Hooks</h2>
					<p class="text-sm text-gray-500">Select hooks to export as JSON</p>
				</div>
			</div>
			<button onclick={onClose} class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
				<X class="w-5 h-5" />
			</button>
		</div>

		<div class="flex-1 flex overflow-hidden">
			<!-- Left: Hook selection -->
			<div class="w-1/2 border-r border-gray-200 dark:border-gray-700 flex flex-col">
				<!-- Selection controls -->
				<div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-800/50 border-b border-gray-200 dark:border-gray-700">
					<span class="text-sm text-gray-600 dark:text-gray-400">
						{selectedIds.size} of {availableHooks.length} selected
					</span>
					<div class="flex gap-2">
						<button onclick={selectAll} class="text-sm text-orange-600 hover:text-orange-700">
							Select all
						</button>
						<span class="text-gray-300">|</span>
						<button onclick={deselectAll} class="text-sm text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">
							Clear
						</button>
					</div>
				</div>

				<!-- Hook list -->
				<div class="flex-1 overflow-auto p-2">
					{#if availableHooks.length === 0}
						<div class="text-center py-8 text-gray-500">
							<p>No hooks to export</p>
						</div>
					{:else}
						<div class="space-y-1">
							{#each availableHooks as hook (hook.id)}
								<button
									onclick={() => toggleHook(hook.id)}
									class="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-left transition-colors
										{selectedIds.has(hook.id) ? 'bg-orange-50 dark:bg-orange-900/20' : ''}"
								>
									{#if selectedIds.has(hook.id)}
										<CheckSquare class="w-5 h-5 text-orange-600 flex-shrink-0" />
									{:else}
										<Square class="w-5 h-5 text-gray-400 flex-shrink-0" />
									{/if}
									<div class="flex-1 min-w-0">
										<p class="font-medium text-gray-900 dark:text-white truncate">{hook.name}</p>
										<p class="text-xs text-gray-500 truncate">
											{hook.eventType}{hook.matcher ? ` (${hook.matcher})` : ''}
										</p>
									</div>
								</button>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<!-- Right: Preview -->
			<div class="w-1/2 flex flex-col">
				<div class="px-4 py-2 bg-gray-50 dark:bg-gray-800/50 border-b border-gray-200 dark:border-gray-700">
					<span class="text-sm font-medium text-gray-700 dark:text-gray-300">Preview</span>
				</div>
				<div class="flex-1 overflow-auto p-4">
					{#if exportPreview}
						<pre class="text-xs font-mono text-gray-700 dark:text-gray-300 whitespace-pre-wrap break-all">{exportPreview}</pre>
					{:else}
						<div class="flex items-center justify-center h-full text-gray-400">
							<p>Select hooks to preview export</p>
						</div>
					{/if}
				</div>
			</div>
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
			<button onclick={onClose} class="btn btn-secondary">
				Cancel
			</button>
			<button
				onclick={copyToClipboard}
				class="btn btn-secondary"
				disabled={selectedIds.size === 0}
			>
				{#if copiedToClipboard}
					<Check class="w-4 h-4 mr-2 text-green-600" />
					Copied!
				{:else}
					<Copy class="w-4 h-4 mr-2" />
					Copy to Clipboard
				{/if}
			</button>
			<button
				onclick={exportToFile}
				class="btn btn-primary"
				disabled={selectedIds.size === 0 || isExporting}
			>
				<Download class="w-4 h-4 mr-2" />
				{isExporting ? 'Exporting...' : 'Export to File'}
			</button>
		</div>
	</div>
</div>
