<script lang="ts">
	import type { SubAgent } from '$lib/types';
	import { agentMemoryLibrary } from '$lib/stores';
	import { projectsStore } from '$lib/stores';
	import { Save, Trash2, X, FolderOpen, Plus } from 'lucide-svelte';

	type Props = {
		subagent: SubAgent;
		onClose: () => void;
	};

	let { subagent, onClose }: Props = $props();

	let selectedScope = $state<'user' | 'project'>('user');
	let showDeleteConfirm = $state(false);

	const projectPath = $derived(projectsStore.selectedProject?.path ?? null);

	$effect(() => {
		agentMemoryLibrary.loadMemory(subagent.name, selectedScope, projectPath);
	});

	async function handleSave() {
		try {
			await agentMemoryLibrary.saveMemory(subagent.name, selectedScope, projectPath);
		} catch (e) {
			console.error('Failed to save agent memory:', e);
		}
	}

	async function handleCreate() {
		agentMemoryLibrary.setContent('');
		try {
			await agentMemoryLibrary.saveMemory(subagent.name, selectedScope, projectPath);
		} catch (e) {
			console.error('Failed to create agent memory:', e);
		}
	}

	async function handleDelete() {
		try {
			await agentMemoryLibrary.deleteMemory(subagent.name, selectedScope, projectPath);
			showDeleteConfirm = false;
		} catch (e) {
			console.error('Failed to delete agent memory:', e);
		}
	}

	function handleScopeChange(scope: 'user' | 'project') {
		selectedScope = scope;
		agentMemoryLibrary.discardChanges();
	}
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
	<div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl max-h-[80vh] flex flex-col">
		<!-- Header -->
		<div class="flex items-center justify-between px-5 py-4 border-b border-gray-200 dark:border-gray-700">
			<div>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">
					Agent Memory — {subagent.name}
				</h2>
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
					Persistent memory for this agent (MEMORY.md)
				</p>
			</div>
			<button
				onclick={onClose}
				class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Scope Tabs -->
		<div class="flex gap-1 px-5 pt-3">
			<button
				onclick={() => handleScopeChange('user')}
				class="px-3 py-1.5 text-sm rounded-lg transition-colors {selectedScope === 'user'
					? 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300 font-medium'
					: 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
			>
				User (~/.claude/)
			</button>
			{#if projectPath}
				<button
					onclick={() => handleScopeChange('project')}
					class="px-3 py-1.5 text-sm rounded-lg transition-colors {selectedScope === 'project'
						? 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300 font-medium'
						: 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'}"
				>
					Project (.claude/)
				</button>
			{/if}
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-5">
			{#if agentMemoryLibrary.isLoading}
				<div class="flex items-center justify-center py-12">
					<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
				</div>
			{:else if agentMemoryLibrary.currentMemory && !agentMemoryLibrary.currentMemory.exists}
				<!-- No memory file exists yet -->
				<div class="text-center py-12">
					<FolderOpen class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
					<h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">No memory file</h3>
					<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
						This agent doesn't have a MEMORY.md file in the {selectedScope} scope yet.
					</p>
					<button onclick={handleCreate} class="btn btn-primary">
						<Plus class="w-4 h-4 mr-2" />
						Create MEMORY.md
					</button>
				</div>
			{:else}
				<!-- Editor -->
				<div class="space-y-3">
					{#if agentMemoryLibrary.currentMemory}
						<div class="flex items-center justify-between">
							<p class="text-xs text-gray-500 dark:text-gray-400 font-mono truncate">
								{agentMemoryLibrary.currentMemory.filePath}
							</p>
							{#if agentMemoryLibrary.currentMemory.lastModified}
								<p class="text-xs text-gray-400 dark:text-gray-500 flex-shrink-0 ml-2">
									{new Date(agentMemoryLibrary.currentMemory.lastModified).toLocaleDateString()}
								</p>
							{/if}
						</div>
					{/if}

					<textarea
						value={agentMemoryLibrary.displayContent}
						oninput={(e) => agentMemoryLibrary.setContent(e.currentTarget.value)}
						class="input w-full font-mono text-sm resize-y"
						rows={16}
						placeholder="# Agent Memory&#10;&#10;- [task](task.md) — description"
					></textarea>
				</div>
			{/if}
		</div>

		<!-- Footer -->
		{#if agentMemoryLibrary.currentMemory?.exists}
			<div class="flex items-center justify-between px-5 py-3 border-t border-gray-200 dark:border-gray-700">
				<div>
					{#if showDeleteConfirm}
						<div class="flex items-center gap-2">
							<span class="text-sm text-red-600 dark:text-red-400">Delete this memory file?</span>
							<button
								onclick={handleDelete}
								class="btn btn-sm bg-red-600 hover:bg-red-700 text-white"
							>
								Confirm
							</button>
							<button
								onclick={() => (showDeleteConfirm = false)}
								class="btn btn-sm"
							>
								Cancel
							</button>
						</div>
					{:else}
						<button
							onclick={() => (showDeleteConfirm = true)}
							class="p-1.5 text-gray-400 hover:text-red-500 dark:hover:text-red-400 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
							title="Delete memory file"
						>
							<Trash2 class="w-4 h-4" />
						</button>
					{/if}
				</div>

				<div class="flex items-center gap-2">
					{#if agentMemoryLibrary.hasUnsavedChanges}
						<button
							onclick={() => agentMemoryLibrary.discardChanges()}
							class="btn btn-sm"
						>
							Discard
						</button>
					{/if}
					<button
						onclick={handleSave}
						disabled={!agentMemoryLibrary.hasUnsavedChanges}
						class="btn btn-primary btn-sm"
					>
						<Save class="w-4 h-4 mr-1.5" />
						Save
					</button>
				</div>
			</div>
		{/if}
	</div>
</div>
