import { invoke } from '@tauri-apps/api/core';
import type { SubAgent, CreateSubAgentRequest, GlobalSubAgent, ProjectSubAgent } from '$lib/types';

class SubAgentLibraryState {
	subagents = $state<SubAgent[]>([]);
	globalSubAgents = $state<GlobalSubAgent[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');

	filteredSubAgents = $derived.by(() => {
		let result = this.subagents;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(a) =>
					a.name.toLowerCase().includes(query) ||
					a.description?.toLowerCase().includes(query) ||
					a.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		return result;
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.subagents = await invoke<SubAgent[]>('get_all_subagents');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load subagents:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadGlobalSubAgents() {
		try {
			this.globalSubAgents = await invoke<GlobalSubAgent[]>('get_global_subagents');
		} catch (e) {
			console.error('Failed to load global subagents:', e);
		}
	}

	async create(request: CreateSubAgentRequest): Promise<SubAgent> {
		const subagent = await invoke<SubAgent>('create_subagent', { subagent: request });
		this.subagents = [...this.subagents, subagent];
		return subagent;
	}

	async update(id: number, request: CreateSubAgentRequest): Promise<SubAgent> {
		const subagent = await invoke<SubAgent>('update_subagent', { id, subagent: request });
		this.subagents = this.subagents.map((a) => (a.id === id ? subagent : a));
		return subagent;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_subagent', { id });
		this.subagents = this.subagents.filter((a) => a.id !== id);
	}

	async addGlobalSubAgent(subagentId: number): Promise<void> {
		await invoke('add_global_subagent', { subagentId });
		await this.loadGlobalSubAgents();
	}

	async removeGlobalSubAgent(subagentId: number): Promise<void> {
		await invoke('remove_global_subagent', { subagentId });
		await this.loadGlobalSubAgents();
	}

	async toggleGlobalSubAgent(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_subagent', { id, enabled });
		await this.loadGlobalSubAgents();
	}

	async assignToProject(projectId: number, subagentId: number): Promise<void> {
		await invoke('assign_subagent_to_project', { projectId, subagentId });
	}

	async removeFromProject(projectId: number, subagentId: number): Promise<void> {
		await invoke('remove_subagent_from_project', { projectId, subagentId });
	}

	async toggleProjectSubAgent(assignmentId: number, enabled: boolean): Promise<void> {
		await invoke('toggle_project_subagent', { assignmentId, enabled });
	}

	async getProjectSubAgents(projectId: number): Promise<ProjectSubAgent[]> {
		return await invoke<ProjectSubAgent[]>('get_project_subagents', { projectId });
	}

	getSubAgentById(id: number): SubAgent | undefined {
		return this.subagents.find((a) => a.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const subagentLibrary = new SubAgentLibraryState();
