import { invoke } from '@tauri-apps/api/core';
import type { AllMemoryFiles, MemoryFileInfo, MemoryScope } from '$lib/types';

class MemoryLibraryState {
	memoryFiles = $state<AllMemoryFiles | null>(null);
	isLoading = $state(false);
	error = $state<string | null>(null);
	selectedScope = $state<MemoryScope>('user');
	projectPath = $state<string | null>(null);
	editedContent = $state<string | null>(null);
	showPreview = $state(false);
	previewHtml = $state('');

	currentFile = $derived.by(() => {
		if (!this.memoryFiles) return null;
		switch (this.selectedScope) {
			case 'user':
				return this.memoryFiles.user;
			case 'project':
				return this.memoryFiles.project ?? null;
			case 'local':
				return this.memoryFiles.local ?? null;
		}
	});

	displayContent = $derived.by(() => {
		if (this.editedContent !== null) return this.editedContent;
		return this.currentFile?.content ?? '';
	});

	hasUnsavedChanges = $derived.by(() => {
		if (this.editedContent === null) return false;
		return this.editedContent !== (this.currentFile?.content ?? '');
	});

	async load() {
		console.log('[memoryLibrary] Loading memory files...');
		this.isLoading = true;
		this.error = null;
		try {
			this.memoryFiles = await invoke<AllMemoryFiles>('get_all_memory_files', {
				projectPath: this.projectPath
			});
			// Reset edited content when loading fresh data
			this.editedContent = null;
			this.previewHtml = '';
			console.log('[memoryLibrary] Loaded memory files');
		} catch (e) {
			this.error = String(e);
			console.error('[memoryLibrary] Failed to load memory files:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async save() {
		if (this.editedContent === null) return;
		console.log(`[memoryLibrary] Saving memory file for scope=${this.selectedScope}`);
		try {
			const updated = await invoke<MemoryFileInfo>('save_memory_file', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				content: this.editedContent
			});
			// Update the stored file info
			if (this.memoryFiles) {
				switch (this.selectedScope) {
					case 'user':
						this.memoryFiles.user = updated;
						break;
					case 'project':
						this.memoryFiles.project = updated;
						break;
					case 'local':
						this.memoryFiles.local = updated;
						break;
				}
			}
			this.editedContent = null;
			console.log('[memoryLibrary] Saved memory file');
		} catch (e) {
			console.error('[memoryLibrary] Failed to save memory file:', e);
			throw e;
		}
	}

	async createFile(content?: string) {
		console.log(`[memoryLibrary] Creating memory file for scope=${this.selectedScope}`);
		try {
			const created = await invoke<MemoryFileInfo>('create_memory_file', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				content: content ?? null
			});
			if (this.memoryFiles) {
				switch (this.selectedScope) {
					case 'user':
						this.memoryFiles.user = created;
						break;
					case 'project':
						this.memoryFiles.project = created;
						break;
					case 'local':
						this.memoryFiles.local = created;
						break;
				}
			}
			this.editedContent = null;
			console.log('[memoryLibrary] Created memory file');
		} catch (e) {
			console.error('[memoryLibrary] Failed to create memory file:', e);
			throw e;
		}
	}

	async deleteFile() {
		console.log(`[memoryLibrary] Deleting memory file for scope=${this.selectedScope}`);
		try {
			await invoke('delete_memory_file', {
				scope: this.selectedScope,
				projectPath: this.projectPath
			});
			// Reload to get fresh state
			await this.load();
			console.log('[memoryLibrary] Deleted memory file');
		} catch (e) {
			console.error('[memoryLibrary] Failed to delete memory file:', e);
			throw e;
		}
	}

	setContent(content: string) {
		this.editedContent = content;
	}

	setScope(scope: MemoryScope) {
		this.selectedScope = scope;
		this.editedContent = null;
		this.previewHtml = '';
	}

	setProjectPath(path: string | null) {
		this.projectPath = path;
		if (!path && this.selectedScope !== 'user') {
			this.selectedScope = 'user';
		}
		this.editedContent = null;
		this.previewHtml = '';
	}

	togglePreview() {
		this.showPreview = !this.showPreview;
		if (this.showPreview) {
			this.renderPreview();
		}
	}

	discardChanges() {
		this.editedContent = null;
		this.previewHtml = '';
	}

	async renderPreview() {
		const content = this.displayContent;
		if (!content) {
			this.previewHtml = '';
			return;
		}
		try {
			this.previewHtml = await invoke<string>('render_markdown', { content });
		} catch (e) {
			console.error('[memoryLibrary] Failed to render markdown:', e);
			this.previewHtml = '<p class="text-red-500">Failed to render preview</p>';
		}
	}
}

export const memoryLibrary = new MemoryLibraryState();
