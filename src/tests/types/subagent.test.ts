import { describe, it, expect } from 'vitest';
import type { ProjectSubAgent, GlobalSubAgent } from '$lib/types/subagent';

describe('SubAgent Types', () => {
	describe('ProjectSubAgent', () => {
		it('should define required fields', () => {
			const projectSubAgent: ProjectSubAgent = {
				id: 1,
				subagentId: 2,
				subagent: {
					id: 2,
					name: 'test-subagent',
					description: 'Test subagent',
					content: 'test content',
					isFavorite: false,
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true
			};

			expect(projectSubAgent.id).toBe(1);
			expect(projectSubAgent.subagentId).toBe(2);
			expect(projectSubAgent.isEnabled).toBe(true);
		});
	});

	describe('GlobalSubAgent', () => {
		it('should define required fields', () => {
			const globalSubAgent: GlobalSubAgent = {
				id: 1,
				subagentId: 2,
				subagent: {
					id: 2,
					name: 'test-subagent',
					description: 'Test subagent',
					content: 'test content',
					isFavorite: false,
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true
			};

			expect(globalSubAgent.id).toBe(1);
			expect(globalSubAgent.subagentId).toBe(2);
			expect(globalSubAgent.isEnabled).toBe(true);
		});
	});
});
