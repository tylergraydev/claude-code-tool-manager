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
	import { i18n } from '$lib/i18n';
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
			notifications.success(i18n.t('memory.saved'));
		} catch {
			notifications.error(i18n.t('memory.saveFailed'));
		}
	}

	async function handleCreate() {
		try {
			await memoryLibrary.createFile();
			notifications.success(i18n.t('memory.created'));
		} catch {
			notifications.error(i18n.t('memory.createFailed'));
		}
	}

	async function handleDelete() {
		showDeleteConfirm = false;
		try {
			await memoryLibrary.deleteFile();
			notifications.success(i18n.t('memory.deleted'));
		} catch {
			notifications.error(i18n.t('memory.deleteFailed'));
		}
	}

	function handleDiscard() {
		memoryLibrary.discardChanges();
		notifications.success(i18n.t('memory.discarded'));
	}

	async function handleRefresh() {
		await memoryLibrary.load();
		notifications.success(i18n.t('memory.refreshed'));
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
	title={i18n.t('page.memory.title')}
	subtitle={i18n.t('page.memory.subtitle')}
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
				<option value="">{i18n.t('common.noProject')}</option>
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
				title={memoryLibrary.showPreview ? i18n.t('memory.hidePreview') : i18n.t('memory.showPreview')}
			>
				{#if memoryLibrary.showPreview}
					<EyeOff class="w-4 h-4 mr-2" />
				{:else}
					<Eye class="w-4 h-4 mr-2" />
				{/if}
				{i18n.t('common.preview')}
			</button>
			<button
				onclick={handleRefresh}
				class="btn btn-ghost"
				title={i18n.t('memory.refreshTitle')}
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
					{i18n.t('common.save')}
				</button>
				<button
					onclick={handleDiscard}
					disabled={!memoryLibrary.hasUnsavedChanges}
					class="btn btn-secondary"
				>
					<Undo2 class="w-4 h-4 mr-2" />
					{i18n.t('common.discard')}
				</button>

				{#if memoryLibrary.hasUnsavedChanges}
					<span class="text-xs text-amber-600 dark:text-amber-400">{i18n.t('memory.unsavedChanges')}</span>
				{/if}

				<div class="flex-1"></div>

				<button
					onclick={() => (showDeleteConfirm = true)}
					class="btn text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
				>
					<Trash2 class="w-4 h-4 mr-2" />
					{i18n.t('memory.deleteFile')}
				</button>
			</div>
		{:else}
			<!-- File doesn't exist — show create prompt -->
			<div class="text-center py-16 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700">
				<div class="text-gray-400 dark:text-gray-500 mb-4">
					<FileText class="w-12 h-12 mx-auto mb-3 opacity-50" />
					<p class="text-lg font-medium">{i18n.t('memory.noFile')}</p>
					<p class="text-sm mt-1">
						{#if memoryLibrary.selectedScope === 'user'}
							{i18n.t('memory.createGlobal')}
						{:else if memoryLibrary.selectedScope === 'project'}
							{i18n.t('memory.createProject')}
						{:else}
							{i18n.t('memory.createLocal')}
						{/if}
					</p>
				</div>
				<button
					onclick={handleCreate}
					class="btn btn-primary"
				>
					<Plus class="w-4 h-4 mr-2" />
					{i18n.t('memory.createFile')}
				</button>
			</div>
		{/if}
	{:else}
		<div class="text-center py-20 text-gray-400 dark:text-gray-500">
			<p>{i18n.t('memory.selectScope')}</p>
		</div>
	{/if}
</div>

<!-- Delete confirmation -->
<ConfirmDialog
	open={showDeleteConfirm}
	title={i18n.t('memory.deleteConfirmTitle')}
	message={i18n.t('memory.deleteConfirm')}
	confirmText={i18n.t('common.delete')}
	variant="danger"
	onConfirm={handleDelete}
	onCancel={() => (showDeleteConfirm = false)}
/>

<!-- Unsaved changes confirmation on scope switch -->
<ConfirmDialog
	open={pendingScopeSwitch !== null}
	title={i18n.t('memory.unsavedTitle')}
	message={i18n.t('memory.unsavedConfirm')}
	confirmText={i18n.t('memory.discardSwitch')}
	variant="warning"
	onConfirm={confirmScopeSwitch}
	onCancel={cancelScopeSwitch}
/>
