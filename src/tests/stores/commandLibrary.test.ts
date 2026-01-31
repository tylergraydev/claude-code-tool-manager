import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Command, GlobalCommand, ProjectCommand } from '$lib/types';

describe('Command Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockCommand = (overrides: Partial<Command> = {}): Command => ({
		id: 1,
		name: 'test-command',
		description: 'A test command',
		content: 'echo "Hello"',
		allowedTools: ['bash'],
		argumentHint: '--help',
		model: 'opus',
		tags: ['test', 'example'],
		source: 'user',
		createdAt: '2024-01-01T00:00:00Z',
		updatedAt: '2024-01-01T00:00:00Z',
		...overrides
	});

	const createMockGlobalCommand = (overrides: Partial<GlobalCommand> = {}): GlobalCommand => ({
		id: 1,
		commandId: 1,
		command: createMockCommand(),
		isEnabled: true,
		...overrides
	});

	const createMockProjectCommand = (overrides: Partial<ProjectCommand> = {}): ProjectCommand => ({
		id: 1,
		commandId: 1,
		command: createMockCommand(),
		isEnabled: true,
		...overrides
	});

	describe('load', () => {
		it('should load commands successfully', async () => {
			const mockCommands = [
				createMockCommand({ id: 1, name: 'command-1' }),
				createMockCommand({ id: 2, name: 'command-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			expect(invoke).toHaveBeenCalledWith('get_all_commands');
			expect(commandLibrary.commands).toHaveLength(2);
			expect(commandLibrary.commands[0].name).toBe('command-1');
			expect(commandLibrary.isLoading).toBe(false);
			expect(commandLibrary.error).toBeNull();
		});

		it('should set isLoading during load', async () => {
			let resolvePromise: (value: Command[]) => void;
			const promise = new Promise<Command[]>((resolve) => {
				resolvePromise = resolve;
			});

			vi.mocked(invoke).mockReturnValueOnce(promise);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			const loadPromise = commandLibrary.load();

			expect(commandLibrary.isLoading).toBe(true);

			resolvePromise!([]);
			await loadPromise;

			expect(commandLibrary.isLoading).toBe(false);
		});

		it('should handle errors during load', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			expect(commandLibrary.error).toBe('Error: Database error');
			expect(commandLibrary.isLoading).toBe(false);
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockCommands = [createMockCommand({ id: 1, name: 'command-1' })];

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
		});
	});

	describe('loadGlobalCommands', () => {
		it('should load global commands successfully', async () => {
			const mockGlobalCommands = [
				createMockGlobalCommand({ id: 1, commandId: 1 }),
				createMockGlobalCommand({ id: 2, commandId: 2 })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.loadGlobalCommands();

			expect(invoke).toHaveBeenCalledWith('get_global_commands');
			expect(commandLibrary.globalCommands).toHaveLength(2);
		});

		it('should handle errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.loadGlobalCommands();

			// Should not throw, just log the error
			expect(commandLibrary.globalCommands).toHaveLength(0);
		});
	});

	describe('create', () => {
		it('should create a new command and add to state', async () => {
			const newCommand = createMockCommand({ id: 3, name: 'new-command' });
			const createRequest = {
				name: 'new-command',
				content: 'echo "New"',
				description: 'A new command'
			};

			vi.mocked(invoke).mockResolvedValueOnce(newCommand);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			const result = await commandLibrary.create(createRequest);

			expect(invoke).toHaveBeenCalledWith('create_command', { command: createRequest });
			expect(result).toEqual(newCommand);
			expect(commandLibrary.commands).toContainEqual(newCommand);
		});

		it('should propagate errors from create', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Create failed'));

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');

			await expect(commandLibrary.create({ name: 'test', content: 'test' })).rejects.toThrow(
				'Create failed'
			);
		});
	});

	describe('update', () => {
		it('should update an existing command', async () => {
			const originalCommand = createMockCommand({ id: 1, name: 'original' });
			const updatedCommand = createMockCommand({ id: 1, name: 'updated' });

			vi.mocked(invoke)
				.mockResolvedValueOnce([originalCommand]) // load
				.mockResolvedValueOnce(updatedCommand); // update

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const result = await commandLibrary.update(1, { name: 'updated', content: 'new content' });

			expect(invoke).toHaveBeenCalledWith('update_command', {
				id: 1,
				command: { name: 'updated', content: 'new content' }
			});
			expect(result.name).toBe('updated');
			expect(commandLibrary.commands.find((c) => c.id === 1)?.name).toBe('updated');
		});

		it('should not modify other commands when updating', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'command-1' }),
				createMockCommand({ id: 2, name: 'command-2' })
			];
			const updatedCommand = createMockCommand({ id: 1, name: 'updated' });

			vi.mocked(invoke)
				.mockResolvedValueOnce(commands)
				.mockResolvedValueOnce(updatedCommand);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();
			await commandLibrary.update(1, { name: 'updated', content: 'test' });

			expect(commandLibrary.commands.find((c) => c.id === 2)?.name).toBe('command-2');
		});
	});

	describe('delete', () => {
		it('should delete a command and remove from state', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'command-1' }),
				createMockCommand({ id: 2, name: 'command-2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(commands) // load
				.mockResolvedValueOnce(undefined); // delete

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			expect(commandLibrary.commands).toHaveLength(2);

			await commandLibrary.delete(1);

			expect(invoke).toHaveBeenCalledWith('delete_command', { id: 1 });
			expect(commandLibrary.commands).toHaveLength(1);
			expect(commandLibrary.commands[0].id).toBe(2);
		});
	});

	describe('addGlobalCommand', () => {
		it('should add a global command and reload', async () => {
			const globalCommands = [createMockGlobalCommand()];

			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // add
				.mockResolvedValueOnce(globalCommands); // reload

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.addGlobalCommand(1);

			expect(invoke).toHaveBeenCalledWith('add_global_command', { commandId: 1 });
			expect(invoke).toHaveBeenCalledWith('get_global_commands');
		});
	});

	describe('removeGlobalCommand', () => {
		it('should remove a global command and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove
				.mockResolvedValueOnce([]); // reload

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.removeGlobalCommand(1);

			expect(invoke).toHaveBeenCalledWith('remove_global_command', { commandId: 1 });
			expect(invoke).toHaveBeenCalledWith('get_global_commands');
		});
	});

	describe('toggleGlobalCommand', () => {
		it('should toggle a global command and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle
				.mockResolvedValueOnce([]); // reload

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.toggleGlobalCommand(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_command', { id: 1, enabled: false });
			expect(invoke).toHaveBeenCalledWith('get_global_commands');
		});
	});

	describe('assignToProject', () => {
		it('should assign command to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_command_to_project', {
				projectId: 1,
				commandId: 2
			});
		});
	});

	describe('removeFromProject', () => {
		it('should remove command from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_command_from_project', {
				projectId: 1,
				commandId: 2
			});
		});
	});

	describe('toggleProjectCommand', () => {
		it('should toggle project command', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.toggleProjectCommand(1, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_command', {
				assignmentId: 1,
				enabled: true
			});
		});
	});

	describe('getProjectCommands', () => {
		it('should get commands for a project', async () => {
			const projectCommands = [
				createMockProjectCommand({ id: 1, commandId: 1 }),
				createMockProjectCommand({ id: 2, commandId: 2 })
			];

			vi.mocked(invoke).mockResolvedValueOnce(projectCommands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			const result = await commandLibrary.getProjectCommands(1);

			expect(invoke).toHaveBeenCalledWith('get_project_commands', { projectId: 1 });
			expect(result).toHaveLength(2);
		});
	});

	describe('getCommandById', () => {
		it('should return correct command by ID', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'command-1' }),
				createMockCommand({ id: 2, name: 'command-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const command = commandLibrary.getCommandById(2);
			expect(command?.name).toBe('command-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			const command = commandLibrary.getCommandById(999);
			expect(command).toBeUndefined();
		});
	});

	describe('setSearch', () => {
		it('should set search query', async () => {
			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');

			commandLibrary.setSearch('test query');

			expect(commandLibrary.searchQuery).toBe('test query');
		});
	});

	describe('filteredCommands', () => {
		it('should filter commands by name', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'angular-cli', description: 'Angular CLI' }),
				createMockCommand({ id: 2, name: 'github-command', description: 'GitHub' }),
				createMockCommand({ id: 3, name: 'huggingface', description: 'HuggingFace' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('github');

			expect(commandLibrary.filteredCommands).toHaveLength(1);
			expect(commandLibrary.filteredCommands[0].name).toBe('github-command');
		});

		it('should filter commands by description', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'cmd-1', description: 'Angular CLI tool' }),
				createMockCommand({ id: 2, name: 'cmd-2', description: 'React framework' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('angular');

			expect(commandLibrary.filteredCommands).toHaveLength(1);
			expect(commandLibrary.filteredCommands[0].description).toContain('Angular');
		});

		it('should filter commands by tags', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'cmd-1', tags: ['javascript', 'frontend'] }),
				createMockCommand({ id: 2, name: 'cmd-2', tags: ['python', 'backend'] }),
				createMockCommand({ id: 3, name: 'cmd-3', tags: ['javascript', 'backend'] })
			];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('javascript');

			expect(commandLibrary.filteredCommands).toHaveLength(2);
		});

		it('should be case-insensitive', async () => {
			const commands = [createMockCommand({ id: 1, name: 'GitHub-CLI' })];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('GITHUB');

			expect(commandLibrary.filteredCommands).toHaveLength(1);
		});

		it('should return all commands when search is empty', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'cmd-1' }),
				createMockCommand({ id: 2, name: 'cmd-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			commandLibrary.setSearch('');

			expect(commandLibrary.filteredCommands).toHaveLength(2);
		});

		it('should handle commands with null/undefined description and tags', async () => {
			const commands = [
				createMockCommand({ id: 1, name: 'test-cmd', description: undefined, tags: undefined })
			];

			vi.mocked(invoke).mockResolvedValueOnce(commands);

			const { commandLibrary } = await import('$lib/stores/commandLibrary.svelte');
			await commandLibrary.load();

			// Should not throw when searching
			commandLibrary.setSearch('something');
			expect(commandLibrary.filteredCommands).toHaveLength(0);

			commandLibrary.setSearch('test');
			expect(commandLibrary.filteredCommands).toHaveLength(1);
		});
	});
});
