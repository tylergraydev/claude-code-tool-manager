import { invoke } from '@tauri-apps/api/core';
import type { Project, CreateProjectRequest, GlobalMcp } from '$lib/types';

class ProjectsState {
	projects = $state<Project[]>([]);
	globalMcps = $state<GlobalMcp[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);

	async loadProjects() {
		this.isLoading = true;
		this.error = null;
		try {
			this.projects = await invoke<Project[]>('get_all_projects');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load projects:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadGlobalMcps() {
		try {
			this.globalMcps = await invoke<GlobalMcp[]>('get_global_mcps');
		} catch (e) {
			console.error('Failed to load global MCPs:', e);
		}
	}

	async addProject(request: CreateProjectRequest): Promise<Project> {
		const project = await invoke<Project>('add_project', { project: request });
		this.projects = [...this.projects, project];
		return project;
	}

	async removeProject(id: number): Promise<void> {
		await invoke('remove_project', { id });
		this.projects = this.projects.filter((p) => p.id !== id);
	}

	async browseForProject(): Promise<string | null> {
		return await invoke<string | null>('browse_for_project');
	}

	async assignMcpToProject(projectId: number, mcpId: number): Promise<void> {
		await invoke('assign_mcp_to_project', { projectId, mcpId });
		await this.loadProjects(); // Reload to get updated assignments
	}

	async removeMcpFromProject(projectId: number, mcpId: number): Promise<void> {
		await invoke('remove_mcp_from_project', { projectId, mcpId });
		await this.loadProjects();
	}

	async toggleProjectMcp(assignmentId: number, enabled: boolean): Promise<void> {
		await invoke('toggle_project_mcp', { assignmentId, enabled });
		await this.loadProjects();
	}

	async syncProjectConfig(projectId: number): Promise<void> {
		await invoke('sync_project_config', { projectId });
	}

	async addGlobalMcp(mcpId: number): Promise<void> {
		await invoke('add_global_mcp', { mcpId });
		await this.loadGlobalMcps();
	}

	async removeGlobalMcp(mcpId: number): Promise<void> {
		await invoke('remove_global_mcp', { mcpId });
		await this.loadGlobalMcps();
	}

	async toggleGlobalMcp(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_mcp_assignment', { id, enabled });
		await this.loadGlobalMcps();
	}

	async syncGlobalConfig(): Promise<void> {
		await invoke('sync_global_config');
	}

	getProjectById(id: number): Project | undefined {
		return this.projects.find((p) => p.id === id);
	}
}

export const projectsStore = new ProjectsState();
