import { invoke } from '@tauri-apps/api/core';
import type { Command, CreateCommandRequest, GlobalCommand, ProjectCommand } from '$lib/types';

class CommandLibraryState {
	commands = $state<Command[]>([]);
	globalCommands = $state<GlobalCommand[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');

	filteredCommands = $derived.by(() => {
		let result = this.commands;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(c) =>
					c.name.toLowerCase().includes(query) ||
					c.description?.toLowerCase().includes(query) ||
					c.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		return result;
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.commands = await invoke<Command[]>('get_all_commands');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load commands:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadGlobalCommands() {
		try {
			this.globalCommands = await invoke<GlobalCommand[]>('get_global_commands');
		} catch (e) {
			console.error('Failed to load global commands:', e);
		}
	}

	async create(request: CreateCommandRequest): Promise<Command> {
		const command = await invoke<Command>('create_command', { command: request });
		this.commands = [...this.commands, command];
		return command;
	}

	async update(id: number, request: CreateCommandRequest): Promise<Command> {
		const command = await invoke<Command>('update_command', { id, command: request });
		this.commands = this.commands.map((c) => (c.id === id ? command : c));
		return command;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_command', { id });
		this.commands = this.commands.filter((c) => c.id !== id);
	}

	async addGlobalCommand(commandId: number): Promise<void> {
		await invoke('add_global_command', { commandId });
		await this.loadGlobalCommands();
	}

	async removeGlobalCommand(commandId: number): Promise<void> {
		await invoke('remove_global_command', { commandId });
		await this.loadGlobalCommands();
	}

	async toggleGlobalCommand(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_command', { id, enabled });
		await this.loadGlobalCommands();
	}

	async assignToProject(projectId: number, commandId: number): Promise<void> {
		await invoke('assign_command_to_project', { projectId, commandId });
	}

	async removeFromProject(projectId: number, commandId: number): Promise<void> {
		await invoke('remove_command_from_project', { projectId, commandId });
	}

	async toggleProjectCommand(assignmentId: number, enabled: boolean): Promise<void> {
		await invoke('toggle_project_command', { assignmentId, enabled });
	}

	async getProjectCommands(projectId: number): Promise<ProjectCommand[]> {
		return await invoke<ProjectCommand[]>('get_project_commands', { projectId });
	}

	getCommandById(id: number): Command | undefined {
		return this.commands.find((c) => c.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const commandLibrary = new CommandLibraryState();
