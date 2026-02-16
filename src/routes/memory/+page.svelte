<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import {
		MemoryScopeSelector,
		MemoryEditor,
		MemoryPreview,
		MemoryFileStatus
	} from '$lib/components/memory';
	import { ConfirmDialog } from '$lib/components/shared';
	import { memoryLibrary, projectsStore, notifications } from '$lib/stores';
	import type { MemoryScope } from '$lib/types';
	import { RefreshCw, FolderOpen, Eye, EyeOff, Plus, Trash2, Save, Undo2, FileText } from 'lucide-svelte';

	let showDeleteConfirm = $state(false);
	let pendingScopeSwitch = $state<MemoryScope | null>(null);

	onMount(async () => {
		await projectsStore.loadProjects();
		await memoryLibrary.load();
	});

	function handleProjectChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		const value = target.value;
		memoryLibrary.setProjectPath(value || null);
		memoryLibrary.load();
	}

	function handleScopeSelect(scope: MemoryScope) {
		if (memoryLibrary.hasUnsavedChanges) {
			pendingScopeSwitch = scope;
		} else {
			memoryLibrary.setScope(scope);
		}
	}

	function confirmScopeSwitch() {
		if (pendingScopeSwitch) {
			memoryLibrary.discardChanges();
			memoryLibrary.setScope(pendingScopeSwitch);
			pendingScopeSwitch = null;
		}
	}

	function cancelScopeSwitch() {
		pendingScopeSwitch = null;
	}

	async function handleSave() {
		try {
			await memoryLibrary.save();
			notifications.success('Memory file saved');
		} catch {
			notifications.error('Failed to save memory file');
		}
	}

	async function handleCreate() {
		try {
			await memoryLibrary.createFile();
			notifications.success('Memory file created');
		} catch {
			notifications.error('Failed to create memory file');
		}
	}

	async function handleDelete() {
		showDeleteConfirm = false;
		try {
			await memoryLibrary.deleteFile();
			notifications.success('Memory file deleted');
		} catch {
			notifications.error('Failed to delete memory file');
		}
	}

	function handleDiscard() {
		memoryLibrary.discardChanges();
		notifications.success('Changes discarded');
	}

	async function handleRefresh() {
		await memoryLibrary.load();
		notifications.success('Memory files refreshed');
	}

	function handleContentChange(content: string) {
		memoryLibrary.setContent(content);
		if (memoryLibrary.showPreview) {
			memoryLibrary.renderPreview();
		}
	}

	function handleTogglePreview() {
		memoryLibrary.togglePreview();
	}
</script>

<Header
	title="Memory Files"
	subtitle="Manage CLAUDE.md memory files — persistent instructions loaded at startup"
/>

<div class="flex-1 overflow-auto p-6">
	<!-- Top bar: Project selector + Scope tabs + Actions -->
	<div class="flex flex-wrap items-center gap-4 mb-4">
		<!-- Project selector -->
		<div class="flex items-center gap-2">
			<FolderOpen class="w-4 h-4 text-gray-500 dark:text-gray-400" />
			<select
				value={memoryLibrary.projectPath ?? ''}
				onchange={handleProjectChange}
				class="input text-sm"
			>
				<option value="">No project</option>
				{#each projectsStore.projects as project}
					<option value={project.path}>{project.name}</option>
				{/each}
			</select>
		</div>

		<!-- Scope selector -->
		<div class="flex-1 min-w-[300px]">
			<MemoryScopeSelector
				selectedScope={memoryLibrary.selectedScope}
				memoryFiles={memoryLibrary.memoryFiles}
				hasProject={!!memoryLibrary.projectPath}
				onselect={handleScopeSelect}
			/>
		</div>

		<!-- Actions -->
		<div class="flex items-center gap-2">
			<button
				onclick={handleTogglePreview}
				class="btn btn-secondary"
				title={memoryLibrary.showPreview ? 'Hide preview' : 'Show preview'}
			>
				{#if memoryLibrary.showPreview}
					<EyeOff class="w-4 h-4 mr-2" />
				{:else}
					<Eye class="w-4 h-4 mr-2" />
				{/if}
				Preview
			</button>
			<button
				onclick={handleRefresh}
				class="btn btn-ghost"
				title="Refresh from disk"
			>
				<RefreshCw class="w-4 h-4" />
			</button>
		</div>
	</div>

	{#if memoryLibrary.isLoading}
		<div class="flex items-center justify-center py-20">
			<div class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"></div>
		</div>
	{:else if memoryLibrary.error}
		<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400">
			{memoryLibrary.error}
		</div>
	{:else if memoryLibrary.currentFile}
		<!-- File status bar -->
		<div class="mb-4">
			<MemoryFileStatus file={memoryLibrary.currentFile} />
		</div>

		{#if memoryLibrary.currentFile.exists}
			<!-- Editor / Preview area -->
			<div class="mb-4 {memoryLibrary.showPreview ? 'grid grid-cols-2 gap-4' : ''}">
				<MemoryEditor
					content={memoryLibrary.displayContent}
					onchange={handleContentChange}
				/>
				{#if memoryLibrary.showPreview}
					<MemoryPreview html={memoryLibrary.previewHtml} />
				{/if}
			</div>

			<!-- Action bar -->
			<div class="flex items-center gap-3">
				<button
					onclick={handleSave}
					disabled={!memoryLibrary.hasUnsavedChanges}
					class="btn btn-primary"
				>
					<Save class="w-4 h-4 mr-2" />
					Save
				</button>
				<button
					onclick={handleDiscard}
					disabled={!memoryLibrary.hasUnsavedChanges}
					class="btn btn-secondary"
				>
					<Undo2 class="w-4 h-4 mr-2" />
					Discard
				</button>

				{#if memoryLibrary.hasUnsavedChanges}
					<span class="text-xs text-amber-600 dark:text-amber-400">Unsaved changes</span>
				{/if}

				<div class="flex-1"></div>

				<button
					onclick={() => (showDeleteConfirm = true)}
					class="btn text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
				>
					<Trash2 class="w-4 h-4 mr-2" />
					Delete File
				</button>
			</div>
		{:else}
			<!-- File doesn't exist — show create prompt -->
			<div class="text-center py-16 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700">
				<div class="text-gray-400 dark:text-gray-500 mb-4">
					<FileText class="w-12 h-12 mx-auto mb-3 opacity-50" />
					<p class="text-lg font-medium">No memory file found</p>
					<p class="text-sm mt-1">
						{#if memoryLibrary.selectedScope === 'user'}
							Create a global CLAUDE.md file with instructions that apply to all projects.
						{:else if memoryLibrary.selectedScope === 'project'}
							Create a project CLAUDE.md file with instructions shared with your team.
						{:else}
							Create a local CLAUDE.local.md file for personal overrides (not committed to git).
						{/if}
					</p>
				</div>
				<button
					onclick={handleCreate}
					class="btn btn-primary"
				>
					<Plus class="w-4 h-4 mr-2" />
					Create File
				</button>
			</div>
		{/if}
	{:else}
		<div class="text-center py-20 text-gray-400 dark:text-gray-500">
			<p>Select a scope to view memory files</p>
		</div>
	{/if}
</div>

<!-- Delete confirmation -->
<ConfirmDialog
	open={showDeleteConfirm}
	title="Delete Memory File"
	message="Are you sure you want to delete this memory file? This action cannot be undone."
	confirmText="Delete"
	variant="danger"
	onConfirm={handleDelete}
	onCancel={() => (showDeleteConfirm = false)}
/>

<!-- Unsaved changes confirmation on scope switch -->
<ConfirmDialog
	open={pendingScopeSwitch !== null}
	title="Unsaved Changes"
	message="You have unsaved changes. Switching scopes will discard them. Continue?"
	confirmText="Discard & Switch"
	variant="warning"
	onConfirm={confirmScopeSwitch}
	onCancel={cancelScopeSwitch}
/>
