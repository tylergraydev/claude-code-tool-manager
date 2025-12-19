import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Projects Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('loadProjects', () => {
		it('should load projects without duplicates', async () => {
			const mockProjects = [
				{
					id: 1,
					name: 'project-1',
					path: 'C:/Code/project-1',
					assignedMcps: []
				},
				{
					id: 2,
					name: 'project-2',
					path: 'C:/Code/project-2',
					assignedMcps: []
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');
			await projectsStore.loadProjects();

			expect(projectsStore.projects).toHaveLength(2);
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockProjects = [
				{
					id: 1,
					name: 'project-1',
					path: 'C:/Code/project-1',
					assignedMcps: []
				}
			];

			vi.mocked(invoke).mockResolvedValue(mockProjects);

			const { projectsStore } = await import('$lib/stores/projects.svelte');

			// Load multiple times
			await projectsStore.loadProjects();
			await projectsStore.loadProjects();
			await projectsStore.loadProjects();

			// Should still only have 1 project
			expect(projectsStore.projects).toHaveLength(1);
		});

		it('should load projects with assigned MCPs', async () => {
			const mockProjects = [
				{
					id: 1,
					name: 'project-1',
					path: 'C:/Code/project-1',
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
	});

	describe('getProjectById', () => {
		it('should return correct project by ID', async () => {
			const mockProjects = [
				{ id: 1, name: 'project-1', path: 'C:/Code/project-1', assignedMcps: [] },
				{ id: 2, name: 'project-2', path: 'C:/Code/project-2', assignedMcps: [] }
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

	describe('globalMcps', () => {
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
	});
});
