import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Project, GlobalMcp } from '$lib/types';

// Helper to create mock projects
const createMockProject = (overrides: Partial<Project> = {}): Project => ({
	id: 1,
	name: 'test-project',
	path: '/path/to/project',
	assignedMcps: [],
	...overrides
});

// Helper to create mock global MCPs
const createMockGlobalMcp = (overrides: Partial<GlobalMcp> = {}): GlobalMcp => ({
	id: 1,
	mcpId: 1,
	isEnabled: true,
	mcp: { id: 1, name: 'test-mcp', type: 'stdio' } as GlobalMcp['mcp'],
	...overrides
});

describe('Projects Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { projectsStore } = await import('$lib/stores/projects.svelte');

			expect(projectsStore.projects).toEqual([]);
			expect(projectsStore.globalMcps).toEqual([]);
			expect(projectsStore.isLoading).toBe(false);
			expect(projectsStore.error).toBeNull();
		});
	});

	describe('loadProjects', () => {
		it('should load projects successfully', async () => {
			const mockProjects = [
				createMockProject({ id: 1, name: 'project-1' }),
				createMockProject({ id: 2, name: 'project-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(invoke).toHaveBeenCalledWith('get_all_projects');
			expect(projectsStore.projects).toEqual(mockProjects);
			expect(projectsStore.isLoading).toBe(false);
			expect(projectsStore.error).toBeNull();
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: Project[]) => void;
			const pendingInvoke = new Promise<Project[]>((resolve) => {
				resolveInvoke = resolve;
			});

			vi.mocked(invoke).mockReturnValueOnce(pendingInvoke);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const loadPromise = projectsStore.loadProjects();

			expect(projectsStore.isLoading).toBe(true);

			resolveInvoke!([]);
			await loadPromise;

			expect(projectsStore.isLoading).toBe(false);
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockProjects = [createMockProject({ id: 1, name: 'project-1' })];

			vi.mocked(invoke).mockResolvedValue(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');

			await projectsStore.loadProjects();
			await projectsStore.loadProjects();
			await projectsStore.loadProjects();

			expect(projectsStore.projects).toHaveLength(1);
		});

		it('should load projects with assigned MCPs', async () => {
			const mockProjects = [
				createMockProject({
					id: 1,
					name: 'project-1',
					assignedMcps: [
						{ id: 1, mcpId: 1, isEnabled: true, mcp: { id: 1, name: 'mcp-1', type: 'stdio' } },
						{ id: 2, mcpId: 2, isEnabled: false, mcp: { id: 2, name: 'mcp-2', type: 'http' } }
					] as Project['assignedMcps']
				})
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.projects[0].assignedMcps).toHaveLength(2);
			expect(projectsStore.projects[0].assignedMcps[0].isEnabled).toBe(true);
			expect(projectsStore.projects[0].assignedMcps[1].isEnabled).toBe(false);
		});

		it('should handle errors and set error state', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.error).toBe('Error: Failed to load');
			expect(projectsStore.isLoading).toBe(false);
		});

		it('should handle non-Error rejection', async () => {
			vi.mocked(invoke).mockRejectedValueOnce('String error');

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.error).toBe('String error');
			expect(projectsStore.isLoading).toBe(false);
		});

		it('should clear previous error on successful load', async () => {
			vi.mocked(invoke)
				.mockRejectedValueOnce(new Error('First error'))
				.mockResolvedValueOnce([]);

			const { projectsStore } = await import('$lib/stores/projects.svelte');

			await projectsStore.loadProjects();
			expect(projectsStore.error).not.toBeNull();

			await projectsStore.loadProjects();
			expect(projectsStore.error).toBeNull();
		});
	});

	describe('loadGlobalMcps', () => {
		it('should load global MCPs successfully', async () => {
			const mockGlobalMcps = [
				createMockGlobalMcp({ id: 1, mcpId: 1 }),
				createMockGlobalMcp({ id: 2, mcpId: 2 })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalMcps);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadGlobalMcps();

			expect(invoke).toHaveBeenCalledWith('get_global_mcps');
			expect(projectsStore.globalMcps).toEqual(mockGlobalMcps);
		});

		it('should not duplicate global MCPs on multiple loads', async () => {
			const mockGlobalMcps = [createMockGlobalMcp({ id: 1 })];

			vi.mocked(invoke).mockResolvedValue(mockGlobalMcps);

			const { projectsStore } = await import('$lib/stores/projects.svelte');

			await projectsStore.loadGlobalMcps();
			await projectsStore.loadGlobalMcps();
			await projectsStore.loadGlobalMcps();

			expect(projectsStore.globalMcps).toHaveLength(1);
		});

		it('should handle errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load global MCPs'));

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadGlobalMcps();

			// Should not throw, just logs error
			expect(projectsStore.globalMcps).toEqual([]);
		});
	});

	describe('addProject', () => {
		it('should add project and update state', async () => {
			const newProject = createMockProject({ id: 3, name: 'new-project' });
			vi.mocked(invoke).mockResolvedValueOnce(newProject);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const result = await projectsStore.addProject({ name: 'new-project', path: '/path/to/new' });

			expect(invoke).toHaveBeenCalledWith('add_project', {
				project: { name: 'new-project', path: '/path/to/new' }
			});
			expect(result).toEqual(newProject);
			expect(projectsStore.projects).toContainEqual(newProject);
		});

		it('should append to existing projects', async () => {
			const existingProject = createMockProject({ id: 1, name: 'existing' });
			const newProject = createMockProject({ id: 2, name: 'new' });

			vi.mocked(invoke)
				.mockResolvedValueOnce([existingProject]) // loadProjects
				.mockResolvedValueOnce(newProject); // addProject

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();
			await projectsStore.addProject({ name: 'new', path: '/path' });

			expect(projectsStore.projects).toHaveLength(2);
			expect(projectsStore.projects[0]).toEqual(existingProject);
			expect(projectsStore.projects[1]).toEqual(newProject);
		});
	});

	describe('removeProject', () => {
		it('should remove project and update state', async () => {
			const projects = [
				createMockProject({ id: 1, name: 'project-1' }),
				createMockProject({ id: 2, name: 'project-2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(projects) // loadProjects
				.mockResolvedValueOnce(undefined); // removeProject

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.projects).toHaveLength(2);

			await projectsStore.removeProject(1);

			expect(invoke).toHaveBeenCalledWith('remove_project', { id: 1 });
			expect(projectsStore.projects).toHaveLength(1);
			expect(projectsStore.projects[0].id).toBe(2);
		});

		it('should handle removing non-existent project', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // loadProjects
				.mockResolvedValueOnce(undefined); // removeProject

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			await projectsStore.removeProject(999);

			expect(invoke).toHaveBeenCalledWith('remove_project', { id: 999 });
			expect(projectsStore.projects).toHaveLength(0);
		});
	});

	describe('browseForProject', () => {
		it('should return selected path', async () => {
			vi.mocked(invoke).mockResolvedValueOnce('/selected/path');

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const result = await projectsStore.browseForProject();

			expect(invoke).toHaveBeenCalledWith('browse_for_project');
			expect(result).toBe('/selected/path');
		});

		it('should return null when cancelled', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(null);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			const result = await projectsStore.browseForProject();

			expect(result).toBeNull();
		});
	});

	describe('assignMcpToProject', () => {
		it('should assign MCP and reload projects', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // assignMcpToProject
				.mockResolvedValueOnce([]); // loadProjects

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.assignMcpToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_mcp_to_project', {
				projectId: 1,
				mcpId: 2
			});
			expect(invoke).toHaveBeenCalledWith('get_all_projects');
		});
	});

	describe('removeMcpFromProject', () => {
		it('should remove MCP from project and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // removeMcpFromProject
				.mockResolvedValueOnce([]); // loadProjects

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.removeMcpFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_mcp_from_project', {
				projectId: 1,
				mcpId: 2
			});
			expect(invoke).toHaveBeenCalledWith('get_all_projects');
		});
	});

	describe('toggleProjectMcp', () => {
		it('should toggle project MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggleProjectMcp
				.mockResolvedValueOnce([]); // loadProjects

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.toggleProjectMcp(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_project_mcp', {
				assignmentId: 1,
				enabled: false
			});
			expect(invoke).toHaveBeenCalledWith('get_all_projects');
		});

		it('should handle enabling', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined)
				.mockResolvedValueOnce([]);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.toggleProjectMcp(5, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_mcp', {
				assignmentId: 5,
				enabled: true
			});
		});
	});

	describe('syncProjectConfig', () => {
		it('should sync project config', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.syncProjectConfig(1);

			expect(invoke).toHaveBeenCalledWith('sync_project_config', { projectId: 1 });
		});
	});

	describe('addGlobalMcp', () => {
		it('should add global MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // addGlobalMcp
				.mockResolvedValueOnce([]); // loadGlobalMcps

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.addGlobalMcp(5);

			expect(invoke).toHaveBeenCalledWith('add_global_mcp', { mcpId: 5 });
			expect(invoke).toHaveBeenCalledWith('get_global_mcps');
		});
	});

	describe('removeGlobalMcp', () => {
		it('should remove global MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // removeGlobalMcp
				.mockResolvedValueOnce([]); // loadGlobalMcps

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.removeGlobalMcp(5);

			expect(invoke).toHaveBeenCalledWith('remove_global_mcp', { mcpId: 5 });
			expect(invoke).toHaveBeenCalledWith('get_global_mcps');
		});
	});

	describe('toggleGlobalMcp', () => {
		it('should toggle global MCP and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggleGlobalMcp
				.mockResolvedValueOnce([]); // loadGlobalMcps

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.toggleGlobalMcp(3, true);

			expect(invoke).toHaveBeenCalledWith('toggle_global_mcp_assignment', { id: 3, enabled: true });
			expect(invoke).toHaveBeenCalledWith('get_global_mcps');
		});

		it('should handle disabling', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined)
				.mockResolvedValueOnce([]);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.toggleGlobalMcp(7, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_mcp_assignment', { id: 7, enabled: false });
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

	describe('getProjectById', () => {
		it('should return correct project by ID', async () => {
			const mockProjects = [
				createMockProject({ id: 1, name: 'project-1' }),
				createMockProject({ id: 2, name: 'project-2' })
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

		it('should return undefined when projects not loaded', async () => {
			const { projectsStore } = await import('$lib/stores/projects.svelte');

			const project = projectsStore.getProjectById(1);
			expect(project).toBeUndefined();
		});
	});
});
