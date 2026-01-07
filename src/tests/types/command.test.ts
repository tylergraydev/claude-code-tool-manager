import { describe, it, expect } from 'vitest';
import type { ProjectCommand, GlobalCommand } from '$lib/types/command';

describe('Command Types', () => {
	describe('ProjectCommand', () => {
		it('should define required fields', () => {
			const projectCommand: ProjectCommand = {
				id: 1,
				commandId: 2,
				command: {
					id: 1,
					name: 'test',
					content: 'echo test',
					isFavorite: false,
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true
			};

			expect(projectCommand.id).toBe(1);
			expect(projectCommand.commandId).toBe(2);
			expect(projectCommand.isEnabled).toBe(true);
		});
	});

	describe('GlobalCommand', () => {
		it('should define required fields', () => {
			const globalCommand: GlobalCommand = {
				id: 1,
				commandId: 2,
				command: {
					id: 1,
					name: 'test',
					content: 'echo test',
					isFavorite: false,
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true
			};

			expect(globalCommand.id).toBe(1);
			expect(globalCommand.commandId).toBe(2);
			expect(globalCommand.isEnabled).toBe(true);
		});
	});
});
