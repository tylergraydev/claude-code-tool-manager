import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Command Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load commands successfully', async () => {
			const mockCommands = [
				{
					id: 1,
					name: 'commit',
					description: 'Create git commits',
					content: 'git commit -m "$message"',
					isFavorite: true,
					tags: ['git', 'version-control'],
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				{
					id: 2,
					name: 'format',
					description: 'Format code',
					content: 'prettier --write "$file"',
					isFavorite: false,
					tags: ['formatting'],
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			expect(commandLibrary.commands).toHaveLength(2);
			expect(commandLibrary.commands[0].name).toBe('commit');
			expect(commandLibrary.isLoading).toBe(false);
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockCommands = [
				{ id: 1, name: 'test-cmd', content: 'echo test', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValue(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');

			await commandLibrary.load();
			await commandLibrary.load();
			await commandLibrary.load();

			expect(commandLibrary.commands).toHaveLength(1);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			expect(commandLibrary.commands).toHaveLength(0);
			expect(commandLibrary.isLoading).toBe(false);
		});

		it('should set isLoading during load', async () => {
			const mockCommands = [{ id: 1, name: 'test', content: 'echo test', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }];

			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			const loadPromise = commandLibrary.load();

			expect(commandLibrary.isLoading).toBe(true);

			resolveInvoke!(mockCommands);
			await loadPromise;

			expect(commandLibrary.isLoading).toBe(false);
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			expect(commandLibrary.error).toContain('Database error');
			expect(commandLibrary.isLoading).toBe(false);
		});
	});

	describe('getCommandById', () => {
		it('should return correct command by ID', async () => {
			const mockCommands = [
				{ id: 1, name: 'cmd-1', content: 'echo 1', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'cmd-2', content: 'echo 2', isFavorite: true, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const command = commandLibrary.getCommandById(2);
			expect(command?.name).toBe('cmd-2');
			expect(command?.isFavorite).toBe(true);
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const command = commandLibrary.getCommandById(999);
			expect(command).toBeUndefined();
		});
	});

	describe('filtering and search', () => {
		it('should filter commands by name', async () => {
			const mockCommands = [
				{ id: 1, name: 'git-commit', content: 'git commit', isFavorite: false, tags: ['git'], source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'npm-install', content: 'npm install', isFavorite: false, tags: ['npm'], source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 3, name: 'format-code', content: 'prettier', isFavorite: false, tags: ['format'], source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('git');

			expect(commandLibrary.filteredCommands).toHaveLength(1);
			expect(commandLibrary.filteredCommands[0].name).toBe('git-commit');
		});

		it('should filter commands by description', async () => {
			const mockCommands = [
				{ id: 1, name: 'cmd1', description: 'Git commit helper', content: 'git', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'cmd2', description: 'Code formatter', content: 'prettier', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('format');

			expect(commandLibrary.filteredCommands).toHaveLength(1);
			expect(commandLibrary.filteredCommands[0].name).toBe('cmd2');
		});

		it('should filter commands by tags', async () => {
			const mockCommands = [
				{ id: 1, name: 'cmd1', content: 'echo 1', isFavorite: false, tags: ['git', 'version-control'], source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'cmd2', content: 'echo 2', isFavorite: false, tags: ['formatting'], source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 3, name: 'cmd3', content: 'echo 3', isFavorite: false, tags: ['git', 'automation'], source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('git');

			expect(commandLibrary.filteredCommands).toHaveLength(2);
		});

		it('should sort favorites first, then alphabetically', async () => {
			const mockCommands = [
				{ id: 1, name: 'zebra', content: 'echo z', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'apple', content: 'echo a', isFavorite: true, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 3, name: 'banana', content: 'echo b', isFavorite: true, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 4, name: 'cherry', content: 'echo c', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const names = commandLibrary.filteredCommands.map((c) => c.name);
			expect(names).toEqual(['apple', 'banana', 'cherry', 'zebra']);
		});

		it('should be case-insensitive', async () => {
			const mockCommands = [
				{ id: 1, name: 'GitCommit', content: 'git', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('GITCOMMIT');

			expect(commandLibrary.filteredCommands).toHaveLength(1);
		});

		it('should return all commands when search is empty', async () => {
			const mockCommands = [
				{ id: 1, name: 'cmd1', content: 'echo 1', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'cmd2', content: 'echo 2', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('');

			expect(commandLibrary.filteredCommands).toHaveLength(2);
		});
	});

	describe('CRUD operations', () => {
		it('should create a command and add to list', async () => {
			const newCommand = {
				id: 3,
				name: 'new-command',
				description: 'New Command',
				content: 'echo new',
				isFavorite: false,
				tags: [],
				source: 'user',
				createdAt: '2024-01-01',
				updatedAt: '2024-01-01'
			};

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // initial load
				.mockResolvedValueOnce(newCommand); // create

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const result = await commandLibrary.create({
				name: 'new-command',
				description: 'New Command',
				content: 'echo new',
				tags: []
			});

			expect(result.id).toBe(3);
			expect(commandLibrary.commands).toHaveLength(1);
			expect(commandLibrary.commands[0].name).toBe('new-command');
			expect(invoke).toHaveBeenCalledWith('create_command', { command: expect.any(Object) });
		});

		it('should update a command in the list', async () => {
			const mockCommands = [{ id: 1, name: 'old-name', content: 'echo old', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }];
			const updatedCommand = { id: 1, name: 'new-name', content: 'echo new', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' };

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockCommands)
				.mockResolvedValueOnce(updatedCommand);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			await commandLibrary.update(1, {
				name: 'new-name',
				description: '',
				content: 'echo new',
				tags: []
			});

			expect(commandLibrary.commands[0].name).toBe('new-name');
			expect(invoke).toHaveBeenCalledWith('update_command', { id: 1, command: expect.any(Object) });
		});

		it('should delete a command from the list', async () => {
			const mockCommands = [
				{ id: 1, name: 'cmd1', content: 'echo 1', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' },
				{ id: 2, name: 'cmd2', content: 'echo 2', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockCommands)
				.mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			await commandLibrary.delete(1);

			expect(commandLibrary.commands).toHaveLength(1);
			expect(commandLibrary.commands[0].id).toBe(2);
			expect(invoke).toHaveBeenCalledWith('delete_command', { id: 1 });
		});
	});

	describe('global commands', () => {
		it('should load global commands', async () => {
			const mockGlobalCommands = [
				{
					id: 1,
					commandId: 1,
					isEnabled: true,
					command: { id: 1, name: 'global-cmd', content: 'echo test', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.loadGlobalCommands();

			expect(commandLibrary.globalCommands).toHaveLength(1);
			expect(invoke).toHaveBeenCalledWith('get_global_commands');
		});

		it('should add global command', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // add_global_command
				.mockResolvedValueOnce([]); // loadGlobalCommands

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.addGlobalCommand(1);

			expect(invoke).toHaveBeenCalledWith('add_global_command', { commandId: 1 });
		});

		it('should remove global command', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove_global_command
				.mockResolvedValueOnce([]); // loadGlobalCommands

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.removeGlobalCommand(1);

			expect(invoke).toHaveBeenCalledWith('remove_global_command', { commandId: 1 });
		});

		it('should toggle global command', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle_global_command
				.mockResolvedValueOnce([]); // loadGlobalCommands

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.toggleGlobalCommand(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_command', { id: 1, enabled: false });
		});
	});

	describe('project commands', () => {
		it('should assign command to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_command_to_project', { projectId: 1, commandId: 2 });
		});

		it('should remove command from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_command_from_project', { projectId: 1, commandId: 2 });
		});

		it('should toggle project command', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.toggleProjectCommand(5, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_command', { assignmentId: 5, enabled: true });
		});

		it('should get project commands', async () => {
			const mockProjectCommands = [
				{
					id: 1,
					commandId: 1,
					isEnabled: true,
					command: { id: 1, name: 'test', content: 'echo', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjectCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			const result = await commandLibrary.getProjectCommands(1);

			expect(result).toHaveLength(1);
			expect(invoke).toHaveBeenCalledWith('get_project_commands', { projectId: 1 });
		});
	});

	describe('state management', () => {
		it('should update command state directly', async () => {
			const mockCommands = [
				{ id: 1, name: 'test', content: 'echo test', isFavorite: false, source: 'user', createdAt: '2024-01-01', updatedAt: '2024-01-01' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const updatedCommand = { ...mockCommands[0], name: 'updated', isFavorite: true };
			commandLibrary.updateCommand(updatedCommand);

			expect(commandLibrary.commands[0].name).toBe('updated');
			expect(commandLibrary.commands[0].isFavorite).toBe(true);
		});

		it('should set search query', async () => {
			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');

			commandLibrary.setSearch('test query');

			expect(commandLibrary.searchQuery).toBe('test query');
		});
	});
});
