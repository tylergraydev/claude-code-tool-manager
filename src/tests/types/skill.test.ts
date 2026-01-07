import { describe, it, expect } from 'vitest';
import type { ProjectSkill, GlobalSkill } from '$lib/types/skill';

describe('Skill Types', () => {
	describe('ProjectSkill', () => {
		it('should define required fields', () => {
			const projectSkill: ProjectSkill = {
				id: 1,
				skillId: 2,
				skill: {
					id: 2,
					name: 'test-skill',
					content: 'test content',
					disableModelInvocation: false,
					isFavorite: false,
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true
			};

			expect(projectSkill.id).toBe(1);
			expect(projectSkill.skillId).toBe(2);
			expect(projectSkill.isEnabled).toBe(true);
		});
	});

	describe('GlobalSkill', () => {
		it('should define required fields', () => {
			const globalSkill: GlobalSkill = {
				id: 1,
				skillId: 2,
				skill: {
					id: 2,
					name: 'test-skill',
					content: 'test content',
					disableModelInvocation: false,
					isFavorite: false,
					source: 'user',
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				isEnabled: true
			};

			expect(globalSkill.id).toBe(1);
			expect(globalSkill.skillId).toBe(2);
			expect(globalSkill.isEnabled).toBe(true);
		});
	});
});
