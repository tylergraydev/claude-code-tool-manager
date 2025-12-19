import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('SubAgent Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('load', () => {
		it('should load subagents', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'code-reviewer', description: 'Reviews code', model: 'haiku' },
				{ id: 2, name: 'doc-writer', description: 'Writes docs', model: 'sonnet' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(subagentLibrary.subagents).toHaveLength(2);
			expect(subagentLibrary.subagents[0].name).toBe('code-reviewer');
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'agent-1', description: 'Desc 1' },
				{ id: 2, name: 'agent-2', description: 'Desc 2' }
			];

			vi.mocked(invoke).mockResolvedValue(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');

			await subagentLibrary.load();
			await subagentLibrary.load();
			await subagentLibrary.load();

			expect(subagentLibrary.subagents).toHaveLength(2);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(subagentLibrary.subagents).toHaveLength(0);
		});

		it('should set isLoading during load', async () => {
			const mockSubAgents = [{ id: 1, name: 'test', description: 'Test' }];

			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			const loadPromise = subagentLibrary.load();

			expect(subagentLibrary.isLoading).toBe(true);

			resolveInvoke!(mockSubAgents);
			await loadPromise;

			expect(subagentLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(subagentLibrary.error).toContain('Database error');
			expect(subagentLibrary.isLoading).toBe(false);
		});
	});

	describe('getSubAgentById', () => {
		it('should return correct subagent by ID', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'agent-1', description: 'Desc 1' },
				{ id: 2, name: 'agent-2', description: 'Desc 2' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			const agent = subagentLibrary.getSubAgentById(2);
			expect(agent?.name).toBe('agent-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			const agent = subagentLibrary.getSubAgentById(999);
			expect(agent).toBeUndefined();
		});
	});

	describe('filtering', () => {
		it('should filter subagents by search query on name', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'code-reviewer', description: 'Reviews code' },
				{ id: 2, name: 'doc-writer', description: 'Writes documentation' },
				{ id: 3, name: 'test-runner', description: 'Runs tests' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('code');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
			expect(subagentLibrary.filteredSubAgents[0].name).toBe('code-reviewer');
		});

		it('should filter subagents by description', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'agent-1', description: 'Python expert' },
				{ id: 2, name: 'agent-2', description: 'JavaScript wizard' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('python');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
			expect(subagentLibrary.filteredSubAgents[0].name).toBe('agent-1');
		});

		it('should filter subagents by tags', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'agent-1', description: 'Desc', tags: ['code', 'review'] },
				{ id: 2, name: 'agent-2', description: 'Desc', tags: ['docs'] },
				{ id: 3, name: 'agent-3', description: 'Desc', tags: ['code', 'testing'] }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('code');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(2);
		});

		it('should be case-insensitive', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'CodeReviewer', description: 'Reviews Code' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('CODEREVIEWER');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
		});

		it('should return all subagents when search is empty', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'agent-1', description: 'Desc 1' },
				{ id: 2, name: 'agent-2', description: 'Desc 2' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(2);
		});
	});

	describe('CRUD operations', () => {
		it('should create a subagent and add to list', async () => {
			const newSubAgent = { id: 3, name: 'new-agent', description: 'New agent', content: 'Content' };

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // initial load
				.mockResolvedValueOnce(newSubAgent); // create

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			const result = await subagentLibrary.create({
				name: 'new-agent',
				description: 'New agent',
				content: 'Content'
			});

			expect(result.id).toBe(3);
			expect(subagentLibrary.subagents).toHaveLength(1);
			expect(subagentLibrary.subagents[0].name).toBe('new-agent');
		});

		it('should update a subagent in the list', async () => {
			const mockSubAgents = [{ id: 1, name: 'old-name', description: 'Old desc' }];
			const updatedSubAgent = { id: 1, name: 'new-name', description: 'New desc' };

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockSubAgents)
				.mockResolvedValueOnce(updatedSubAgent);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			await subagentLibrary.update(1, {
				name: 'new-name',
				description: 'New desc',
				content: ''
			});

			expect(subagentLibrary.subagents[0].name).toBe('new-name');
		});

		it('should delete a subagent from the list', async () => {
			const mockSubAgents = [
				{ id: 1, name: 'agent-1', description: 'Desc 1' },
				{ id: 2, name: 'agent-2', description: 'Desc 2' }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockSubAgents)
				.mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			await subagentLibrary.delete(1);

			expect(subagentLibrary.subagents).toHaveLength(1);
			expect(subagentLibrary.subagents[0].id).toBe(2);
		});
	});

	describe('global subagents', () => {
		it('should load global subagents', async () => {
			const mockGlobalSubAgents = [
				{ id: 1, subagent_id: 1, is_enabled: true, subagent: { id: 1, name: 'global-agent', description: 'Desc' } }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.loadGlobalSubAgents();

			expect(subagentLibrary.globalSubAgents).toHaveLength(1);
		});

		it('should add global subagent', async () => {
			const mockGlobalSubAgents = [
				{ id: 1, subagent_id: 1, is_enabled: true, subagent: { id: 1, name: 'test', description: 'Desc' } }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // add_global_subagent
				.mockResolvedValueOnce(mockGlobalSubAgents); // loadGlobalSubAgents

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.addGlobalSubAgent(1);

			expect(invoke).toHaveBeenCalledWith('add_global_subagent', { subagentId: 1 });
		});

		it('should remove global subagent', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove_global_subagent
				.mockResolvedValueOnce([]); // loadGlobalSubAgents

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.removeGlobalSubAgent(1);

			expect(invoke).toHaveBeenCalledWith('remove_global_subagent', { subagentId: 1 });
		});

		it('should toggle global subagent', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle_global_subagent
				.mockResolvedValueOnce([]); // loadGlobalSubAgents

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.toggleGlobalSubAgent(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_subagent', { id: 1, enabled: false });
		});
	});

	describe('project subagents', () => {
		it('should assign subagent to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_subagent_to_project', { projectId: 1, subagentId: 2 });
		});

		it('should remove subagent from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_subagent_from_project', { projectId: 1, subagentId: 2 });
		});

		it('should toggle project subagent', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.toggleProjectSubAgent(5, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_subagent', { assignmentId: 5, enabled: true });
		});

		it('should get project subagents', async () => {
			const mockProjectSubAgents = [
				{ id: 1, subagent_id: 1, is_enabled: true, subagent: { id: 1, name: 'test', description: 'Desc' } }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjectSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			const result = await subagentLibrary.getProjectSubAgents(1);

			expect(result).toHaveLength(1);
			expect(invoke).toHaveBeenCalledWith('get_project_subagents', { projectId: 1 });
		});
	});
});
