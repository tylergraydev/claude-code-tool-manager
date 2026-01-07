import { describe, it, expect } from 'vitest';
import type { ProjectMcp, GlobalMcp } from '$lib/types/project';

describe('Project Types', () => {
	describe('ProjectMcp', () => {
		it('should define required fields', () => {
			const projectMcp: ProjectMcp = {
				id: 1,
				mcpId: 2,
				mcp: {
					id: 2,
					name: 'test-mcp',
					description: null,
					type: 'stdio',
					command: null,
					args: null,
					url: null,
					headers: null,
					env: null,
					icon: null,
					tags: null,
					source: 'manual',
					sourcePath: null,
					isEnabledGlobal: false,
					isFavorite: false,
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true,
				envOverrides: null,
				displayOrder: 1
			};

			expect(projectMcp.id).toBe(1);
			expect(projectMcp.mcpId).toBe(2);
			expect(projectMcp.isEnabled).toBe(true);
		});
	});

	describe('GlobalMcp', () => {
		it('should define required fields', () => {
			const globalMcp: GlobalMcp = {
				id: 1,
				mcpId: 2,
				mcp: {
					id: 2,
					name: 'test-mcp',
					description: null,
					type: 'stdio',
					command: null,
					args: null,
					url: null,
					headers: null,
					env: null,
					icon: null,
					tags: null,
					source: 'manual',
					sourcePath: null,
					isEnabledGlobal: false,
					isFavorite: false,
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true,
				envOverrides: null
			};

			expect(globalMcp.id).toBe(1);
			expect(globalMcp.mcpId).toBe(2);
			expect(globalMcp.isEnabled).toBe(true);
		});
	});
});
