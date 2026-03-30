import { invoke } from '@tauri-apps/api/core';
import type { AgentMemoryFileInfo, AgentMemoryEntry } from '$lib/types';

class AgentMemoryLibraryState {
	currentMemory = $state<AgentMemoryFileInfo | null>(null);
	entries = $state<AgentMemoryEntry[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	editedContent = $state<string | null>(null);

	displayContent = $derived.by(() => {
		if (this.editedContent !== null) return this.editedContent;
		return this.currentMemory?.content ?? '';
	});

	hasUnsavedChanges = $derived.by(() => {
		if (this.editedContent === null) return false;
		return this.editedContent !== (this.currentMemory?.content ?? '');
	});

	async loadMemory(agentName: string, scope: string, projectPath?: string | null) {
		this.isLoading = true;
		this.error = null;
		try {
			this.currentMemory = await invoke<AgentMemoryFileInfo>('get_agent_memory', {
				agentName,
				scope,
				projectPath: projectPath ?? null
			});
			this.editedContent = null;
		} catch (e) {
			this.error = String(e);
			console.error('[agentMemoryLibrary] Failed to load agent memory:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async saveMemory(agentName: string, scope: string, projectPath?: string | null) {
		if (this.editedContent === null) return;
		try {
			this.currentMemory = await invoke<AgentMemoryFileInfo>('save_agent_memory', {
				agentName,
				scope,
				projectPath: projectPath ?? null,
				content: this.editedContent
			});
			this.editedContent = null;
		} catch (e) {
			console.error('[agentMemoryLibrary] Failed to save agent memory:', e);
			throw e;
		}
	}

	async deleteMemory(agentName: string, scope: string, projectPath?: string | null) {
		try {
			await invoke('delete_agent_memory', {
				agentName,
				scope,
				projectPath: projectPath ?? null
			});
			this.currentMemory = null;
			this.editedContent = null;
		} catch (e) {
			console.error('[agentMemoryLibrary] Failed to delete agent memory:', e);
			throw e;
		}
	}

	async listEntries(scope: string, projectPath?: string | null) {
		try {
			this.entries = await invoke<AgentMemoryEntry[]>('list_agent_memories', {
				scope,
				projectPath: projectPath ?? null
			});
		} catch (e) {
			console.error('[agentMemoryLibrary] Failed to list agent memories:', e);
			this.entries = [];
		}
	}

	setContent(content: string) {
		this.editedContent = content;
	}

	discardChanges() {
		this.editedContent = null;
	}

	clear() {
		this.currentMemory = null;
		this.entries = [];
		this.editedContent = null;
		this.error = null;
	}
}

export const agentMemoryLibrary = new AgentMemoryLibraryState();
