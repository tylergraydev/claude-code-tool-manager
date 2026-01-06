export interface Command {
	id: number;
	name: string;
	description?: string;
	content: string;
	allowedTools?: string[];
	argumentHint?: string;
	model?: string;
	tags?: string[];
	source: string;
	createdAt: string;
	updatedAt: string;
}

export interface CreateCommandRequest {
	name: string;
	description?: string;
	content: string;
	allowedTools?: string[];
	argumentHint?: string;
	model?: string;
	tags?: string[];
}

export interface ProjectCommand {
	id: number;
	commandId: number;
	command: Command;
	isEnabled: boolean;
}

export interface GlobalCommand {
	id: number;
	commandId: number;
	command: Command;
	isEnabled: boolean;
}
