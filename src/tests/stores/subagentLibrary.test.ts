import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { SubAgent, GlobalSubAgent, ProjectSubAgent } from '$lib/types';

describe('SubAgent Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockSubAgent = (overrides: Partial<SubAgent> = {}): SubAgent => ({
		id: 1,
		name: 'test-subagent',
		description: 'A test subagent',
		content: 'You are a helpful assistant',
		tools: ['read', 'write'],
		model: 'opus',
		permissionMode: 'default',
		skills: ['coding', 'writing'],
		tags: ['test', 'example'],
		source: 'user',
		createdAt: '2024-01-01T00:00:00Z',
		updatedAt: '2024-01-01T00:00:00Z',
		...overrides
	});

	const createMockGlobalSubAgent = (overrides: Partial<GlobalSubAgent> = {}): GlobalSubAgent => ({
		id: 1,
		subagentId: 1,
		subagent: createMockSubAgent(),
		isEnabled: true,
		...overrides
	});

	const createMockProjectSubAgent = (
		overrides: Partial<ProjectSubAgent> = {}
	): ProjectSubAgent => ({
		id: 1,
		subagentId: 1,
		subagent: createMockSubAgent(),
		isEnabled: true,
		...overrides
	});

	describe('load', () => {
		it('should load subagents successfully', async () => {
			const mockSubAgents = [
				createMockSubAgent({ id: 1, name: 'subagent-1' }),
				createMockSubAgent({ id: 2, name: 'subagent-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(invoke).toHaveBeenCalledWith('get_all_subagents');
			expect(subagentLibrary.subagents).toHaveLength(2);
			expect(subagentLibrary.subagents[0].name).toBe('subagent-1');
			expect(subagentLibrary.isLoading).toBe(false);
			expect(subagentLibrary.error).toBeNull();
		});

		it('should set isLoading during load', async () => {
			let resolvePromise: (value: SubAgent[]) => void;
			const promise = new Promise<SubAgent[]>((resolve) => {
				resolvePromise = resolve;
			});

			vi.mocked(invoke).mockReturnValueOnce(promise);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			const loadPromise = subagentLibrary.load();

			expect(subagentLibrary.isLoading).toBe(true);

			resolvePromise!([]);
			await loadPromise;

			expect(subagentLibrary.isLoading).toBe(false);
		});

		it('should handle errors during load', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(subagentLibrary.error).toBe('Error: Database error');
			expect(subagentLibrary.isLoading).toBe(false);
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockSubAgents = [createMockSubAgent({ id: 1, name: 'subagent-1' })];

			vi.mocked(invoke).mockResolvedValue(mockSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');

			await subagentLibrary.load();
			await subagentLibrary.load();
			await subagentLibrary.load();

			expect(subagentLibrary.subagents).toHaveLength(1);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(subagentLibrary.subagents).toHaveLength(0);
		});
	});

	describe('loadGlobalSubAgents', () => {
		it('should load global subagents successfully', async () => {
			const mockGlobalSubAgents = [
				createMockGlobalSubAgent({ id: 1, subagentId: 1 }),
				createMockGlobalSubAgent({ id: 2, subagentId: 2 })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.loadGlobalSubAgents();

			expect(invoke).toHaveBeenCalledWith('get_global_subagents');
			expect(subagentLibrary.globalSubAgents).toHaveLength(2);
		});

		it('should handle errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.loadGlobalSubAgents();

			expect(subagentLibrary.globalSubAgents).toHaveLength(0);
		});
	});

	describe('create', () => {
		it('should create a new subagent and add to state', async () => {
			const newSubAgent = createMockSubAgent({ id: 3, name: 'new-subagent' });
			const createRequest = {
				name: 'new-subagent',
				description: 'A new subagent',
				content: 'You are helpful'
			};

			vi.mocked(invoke).mockResolvedValueOnce(newSubAgent);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			const result = await subagentLibrary.create(createRequest);

			expect(invoke).toHaveBeenCalledWith('create_subagent', { subagent: createRequest });
			expect(result).toEqual(newSubAgent);
			expect(subagentLibrary.subagents).toContainEqual(newSubAgent);
		});

		it('should propagate errors from create', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Create failed'));

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');

			await expect(
				subagentLibrary.create({ name: 'test', description: 'test', content: 'test' })
			).rejects.toThrow('Create failed');
		});
	});

	describe('update', () => {
		it('should update an existing subagent', async () => {
			const originalSubAgent = createMockSubAgent({ id: 1, name: 'original' });
			const updatedSubAgent = createMockSubAgent({ id: 1, name: 'updated' });

			vi.mocked(invoke)
				.mockResolvedValueOnce([originalSubAgent])
				.mockResolvedValueOnce(updatedSubAgent);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			const result = await subagentLibrary.update(1, {
				name: 'updated',
				description: 'new desc',
				content: 'new content'
			});

			expect(invoke).toHaveBeenCalledWith('update_subagent', {
				id: 1,
				subagent: { name: 'updated', description: 'new desc', content: 'new content' }
			});
			expect(result.name).toBe('updated');
			expect(subagentLibrary.subagents.find((a) => a.id === 1)?.name).toBe('updated');
		});

		it('should not modify other subagents when updating', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'subagent-1' }),
				createMockSubAgent({ id: 2, name: 'subagent-2' })
			];
			const updatedSubAgent = createMockSubAgent({ id: 1, name: 'updated' });

			vi.mocked(invoke).mockResolvedValueOnce(subagents).mockResolvedValueOnce(updatedSubAgent);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();
			await subagentLibrary.update(1, { name: 'updated', description: 'test', content: 'test' });

			expect(subagentLibrary.subagents.find((a) => a.id === 2)?.name).toBe('subagent-2');
		});
	});

	describe('delete', () => {
		it('should delete a subagent and remove from state', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'subagent-1' }),
				createMockSubAgent({ id: 2, name: 'subagent-2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(subagents)
				.mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			expect(subagentLibrary.subagents).toHaveLength(2);

			await subagentLibrary.delete(1);

			expect(invoke).toHaveBeenCalledWith('delete_subagent', { id: 1 });
			expect(subagentLibrary.subagents).toHaveLength(1);
			expect(subagentLibrary.subagents[0].id).toBe(2);
		});
	});

	describe('addGlobalSubAgent', () => {
		it('should add a global subagent and reload', async () => {
			const globalSubAgents = [createMockGlobalSubAgent()];

			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined)
				.mockResolvedValueOnce(globalSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.addGlobalSubAgent(1);

			expect(invoke).toHaveBeenCalledWith('add_global_subagent', { subagentId: 1 });
			expect(invoke).toHaveBeenCalledWith('get_global_subagents');
		});
	});

	describe('removeGlobalSubAgent', () => {
		it('should remove a global subagent and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined)
				.mockResolvedValueOnce([]);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.removeGlobalSubAgent(1);

			expect(invoke).toHaveBeenCalledWith('remove_global_subagent', { subagentId: 1 });
			expect(invoke).toHaveBeenCalledWith('get_global_subagents');
		});
	});

	describe('toggleGlobalSubAgent', () => {
		it('should toggle a global subagent and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined)
				.mockResolvedValueOnce([]);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.toggleGlobalSubAgent(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_subagent', { id: 1, enabled: false });
			expect(invoke).toHaveBeenCalledWith('get_global_subagents');
		});
	});

	describe('assignToProject', () => {
		it('should assign subagent to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_subagent_to_project', {
				projectId: 1,
				subagentId: 2
			});
		});
	});

	describe('removeFromProject', () => {
		it('should remove subagent from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_subagent_from_project', {
				projectId: 1,
				subagentId: 2
			});
		});
	});

	describe('toggleProjectSubAgent', () => {
		it('should toggle project subagent', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.toggleProjectSubAgent(1, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_subagent', {
				assignmentId: 1,
				enabled: true
			});
		});
	});

	describe('getProjectSubAgents', () => {
		it('should get subagents for a project', async () => {
			const projectSubAgents = [
				createMockProjectSubAgent({ id: 1, subagentId: 1 }),
				createMockProjectSubAgent({ id: 2, subagentId: 2 })
			];

			vi.mocked(invoke).mockResolvedValueOnce(projectSubAgents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			const result = await subagentLibrary.getProjectSubAgents(1);

			expect(invoke).toHaveBeenCalledWith('get_project_subagents', { projectId: 1 });
			expect(result).toHaveLength(2);
		});
	});

	describe('getSubAgentById', () => {
		it('should return correct subagent by ID', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'subagent-1' }),
				createMockSubAgent({ id: 2, name: 'subagent-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			const subagent = subagentLibrary.getSubAgentById(2);
			expect(subagent?.name).toBe('subagent-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			const subagent = subagentLibrary.getSubAgentById(999);
			expect(subagent).toBeUndefined();
		});
	});

	describe('setSearch', () => {
		it('should set search query', async () => {
			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');

			subagentLibrary.setSearch('test query');

			expect(subagentLibrary.searchQuery).toBe('test query');
		});
	});

	describe('filteredSubAgents', () => {
		it('should filter subagents by name', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'code-assistant', description: 'Helps with code' }),
				createMockSubAgent({ id: 2, name: 'writer-bot', description: 'Writing help' }),
				createMockSubAgent({ id: 3, name: 'research-agent', description: 'Research tasks' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('writer');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
			expect(subagentLibrary.filteredSubAgents[0].name).toBe('writer-bot');
		});

		it('should filter subagents by description', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'agent-1', description: 'Helps with Python code' }),
				createMockSubAgent({ id: 2, name: 'agent-2', description: 'JavaScript expert' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('python');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
			expect(subagentLibrary.filteredSubAgents[0].description).toContain('Python');
		});

		it('should filter subagents by tags', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'agent-1', tags: ['javascript', 'frontend'] }),
				createMockSubAgent({ id: 2, name: 'agent-2', tags: ['python', 'backend'] }),
				createMockSubAgent({ id: 3, name: 'agent-3', tags: ['javascript', 'backend'] })
			];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('javascript');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(2);
		});

		it('should be case-insensitive', async () => {
			const subagents = [createMockSubAgent({ id: 1, name: 'CodeAssistant' })];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('CODEASSISTANT');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
		});

		it('should return all subagents when search is empty', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'agent-1' }),
				createMockSubAgent({ id: 2, name: 'agent-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('');

			expect(subagentLibrary.filteredSubAgents).toHaveLength(2);
		});

		it('should handle subagents with undefined description and tags', async () => {
			const subagents = [
				createMockSubAgent({ id: 1, name: 'test-agent', description: '', tags: undefined })
			];

			vi.mocked(invoke).mockResolvedValueOnce(subagents);

			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');
			await subagentLibrary.load();

			subagentLibrary.setSearch('something');
			expect(subagentLibrary.filteredSubAgents).toHaveLength(0);

			subagentLibrary.setSearch('test');
			expect(subagentLibrary.filteredSubAgents).toHaveLength(1);
		});
	});
});
