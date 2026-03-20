import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Projects Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('loadProjects', () => {
		it('should load projects without duplicates', async () => {
			const mockProjects = [
				{ id: 1, name: 'project-1', path: 'C:/Code/project-1', assignedMcps: [], isFavorite: false },
				{ id: 2, name: 'project-2', path: 'C:/Code/project-2', assignedMcps: [], isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.projects).toHaveLength(2);
			expect(projectsStore.isLoading).toBe(false);
			expect(projectsStore.error).toBeNull();
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockProjects = [
				{ id: 1, name: 'project-1', path: 'C:/Code/project-1', assignedMcps: [], isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValue(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();
			await projectsStore.loadProjects();
			await projectsStore.loadProjects();

			expect(projectsStore.projects).toHaveLength(1);
		});

		it('should load projects with assigned MCPs', async () => {
			const mockProjects = [
				{
					id: 1, name: 'project-1', path: 'C:/Code/project-1', isFavorite: false,
					assignedMcps: [
						{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'mcp-1', type: 'stdio' } },
						{ id: 2, mcpId: 2, isEnabled: false, mcp: { id: 2, name: 'mcp-2', type: 'http' } }
					]
				}
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.projects[0].assignedMcps).toHaveLength(2);
			expect(projectsStore.projects[0].assignedMcps[0].isEnabled).toBe(true);
			expect(projectsStore.projects[0].assignedMcps[1].isEnabled).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('DB error'));

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.error).toContain('DB error');
			expect(projectsStore.isLoading).toBe(false);
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const loadPromise = projectsStore.loadProjects();

			expect(projectsStore.isLoading).toBe(true);

			resolveInvoke!([]);
			await loadPromise;

			expect(projectsStore.isLoading).toBe(false);
		});
	});

	describe('loadGlobalMcps', () => {
		it('should load global MCPs without duplicates', async () => {
			const mockGlobalMcps = [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'global-1', type: 'stdio' } },
				{ id: 2, mcpId: 2, isEnabled: true, mcp: { id: 2, name: 'global-2', type: 'http' } }
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalMcps);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadGlobalMcps();

			expect(projectsStore.globalMcps).toHaveLength(2);
		});

		it('should not duplicate global MCPs on multiple loads', async () => {
			const mockGlobalMcps = [
				{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'global-1', type: 'stdio' } }
			];
			vi.mocked(invoke).mockResolvedValue(mockGlobalMcps);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadGlobalMcps();
			await projectsStore.loadGlobalMcps();
			await projectsStore.loadGlobalMcps();

			expect(projectsStore.globalMcps).toHaveLength(1);
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadGlobalMcps();

			// Should not throw, just log error
			expect(projectsStore.globalMcps).toEqual([]);
		});
	});

	describe('addProject', () => {
		it('should add a project and update local state', async () => {
			const newProject = {
				id: 5, name: 'new-project', path: '/code/new-project',
				assignedMcps: [], isFavorite: false
			};
			vi.mocked(invoke).mockResolvedValueOnce(newProject);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const result = await projectsStore.addProject({
				name: 'new-project',
				path: '/code/new-project'
			} as any);

			expect(result.id).toBe(5);
			expect(projectsStore.projects).toHaveLength(1);
			expect(projectsStore.projects[0].name).toBe('new-project');
			expect(invoke).toHaveBeenCalledWith('add_project', {
				project: { name: 'new-project', path: '/code/new-project' }
			});
		});
	});

	describe('removeProject', () => {
		it('should remove a project and update local state', async () => {
			const mockProjects = [
				{ id: 1, name: 'proj-1', path: '/a', assignedMcps: [], isFavorite: false },
				{ id: 2, name: 'proj-2', path: '/b', assignedMcps: [], isFavorite: false }
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(mockProjects) // loadProjects
				.mockResolvedValueOnce(undefined); // remove_project

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();
			await projectsStore.removeProject(1);

			expect(projectsStore.projects).toHaveLength(1);
			expect(projectsStore.projects[0].id).toBe(2);
			expect(invoke).toHaveBeenCalledWith('remove_project', { id: 1 });
		});
	});

	describe('browseForProject', () => {
		it('should return selected path', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('/selected/path');

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const result = await projectsStore.browseForProject();

			expect(result).toBe('/selected/path');
			expect(invoke).toHaveBeenCalledWith('browse_for_project');
		});

		it('should return null when cancelled', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(null);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const result = await projectsStore.browseForProject();

			expect(result).toBeNull();
		});
	});

	describe('getProjectById', () => {
		it('should return correct project by ID', async () => {
			const mockProjects = [
				{ id: 1, name: 'project-1', path: 'C:/Code/project-1', assignedMcps: [], isFavorite: false },
				{ id: 2, name: 'project-2', path: 'C:/Code/project-2', assignedMcps: [], isFavorite: false }
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			const project = projectsStore.getProjectById(2);
			expect(project?.name).toBe('project-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			const project = projectsStore.getProjectById(999);
			expect(project).toBeUndefined();
		});
	});

	describe('assignMcpToProject', () => {
		it('should call invoke and reload projects', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // assign_mcp_to_project
				.mockResolvedValueOnce([]); // loadProjects

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.assignMcpToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_mcp_to_project', {
				projectId: 1,
				mcpId: 2
			});
		});
	});

	describe('removeMcpFromProject', () => {
		it('should call invoke and reload projects', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove_mcp_from_project
				.mockResolvedValueOnce([]); // loadProjects

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.removeMcpFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_mcp_from_project', {
				projectId: 1,
				mcpId: 2
			});
		});
	});

	describe('toggleProjectMcp', () => {
		it('should call invoke with correct parameters', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle_project_mcp
				.mockResolvedValueOnce([]); // loadProjects

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.toggleProjectMcp(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_project_mcp', {
				assignmentId: 1,
				enabled: false
			});
		});
	});

	describe('syncProjectConfig', () => {
		it('should sync project config', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.syncProjectConfig(5);

			expect(invoke).toHaveBeenCalledWith('sync_project_config', { projectId: 5 });
		});
	});

	describe('addGlobalMcp', () => {
		it('should add global MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // add_global_mcp
				.mockResolvedValueOnce([]); // loadGlobalMcps

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.addGlobalMcp(3);

			expect(invoke).toHaveBeenCalledWith('add_global_mcp', { mcpId: 3 });
		});
	});

	describe('removeGlobalMcp', () => {
		it('should remove global MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove_global_mcp
				.mockResolvedValueOnce([]); // loadGlobalMcps

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.removeGlobalMcp(3);

			expect(invoke).toHaveBeenCalledWith('remove_global_mcp', { mcpId: 3 });
		});
	});

	describe('toggleGlobalMcp', () => {
		it('should toggle global MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle_global_mcp_assignment
				.mockResolvedValueOnce([]); // loadGlobalMcps

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.toggleGlobalMcp(1, true);

			expect(invoke).toHaveBeenCalledWith('toggle_global_mcp_assignment', { id: 1, enabled: true });
		});
	});

	describe('syncGlobalConfig', () => {
		it('should sync global config', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.syncGlobalConfig();

			expect(invoke).toHaveBeenCalledWith('sync_global_config');
		});
	});

	describe('toggleFavorite', () => {
		it('should toggle favorite and update local state', async () => {
			const mockProjects = [
				{ id: 1, name: 'proj-1', path: '/a', assignedMcps: [], isFavorite: false },
				{ id: 2, name: 'proj-2', path: '/b', assignedMcps: [], isFavorite: false }
			];
			vi.mocked(invoke)
				.mockResolvedValueOnce(mockProjects) // loadProjects
				.mockResolvedValueOnce(undefined); // toggle_project_favorite

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();
			await projectsStore.toggleFavorite(1, true);

			expect(projectsStore.projects.find((p) => p.id === 1)?.isFavorite).toBe(true);
			expect(projectsStore.projects.find((p) => p.id === 2)?.isFavorite).toBe(false);
			expect(invoke).toHaveBeenCalledWith('toggle_project_favorite', { id: 1, favorite: true });
		});
	});

	describe('sortedProjects', () => {
		it('should sort favorites first, then by name', async () => {
			const mockProjects = [
				{ id: 1, name: 'Zeta', path: '/z', assignedMcps: [], isFavorite: false },
				{ id: 2, name: 'Alpha', path: '/a', assignedMcps: [], isFavorite: true },
				{ id: 3, name: 'Beta', path: '/b', assignedMcps: [], isFavorite: false },
				{ id: 4, name: 'Gamma', path: '/g', assignedMcps: [], isFavorite: true }
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			const sorted = projectsStore.sortedProjects;
			// Favorites first (Alpha, Gamma), then non-favorites (Beta, Zeta)
			expect(sorted[0].name).toBe('Alpha');
			expect(sorted[1].name).toBe('Gamma');
			expect(sorted[2].name).toBe('Beta');
			expect(sorted[3].name).toBe('Zeta');
		});

		it('should return empty array when no projects', async () => {
			const { projectsStore } = await import('$lib/stores/projects.svelte');
			expect(projectsStore.sortedProjects).toEqual([]);
		});
	});
});
