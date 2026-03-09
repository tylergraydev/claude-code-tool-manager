import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Skill Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe('load', () => {
		it('should load skills', async () => {
			const mockSkills = [
				{ id: 1, name: 'commit', description: 'Create commits' },
				{ id: 2, name: 'review', description: 'Review code', skillType: 'skill' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			expect(skillLibrary.skills).toHaveLength(2);
			expect(skillLibrary.skills[0].name).toBe('commit');
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockSkills = [
				{ id: 1, name: 'skill-1' },
				{ id: 2, name: 'skill-2', skillType: 'skill' }
			];

			vi.mocked(invoke).mockResolvedValue(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');

			await skillLibrary.load();
			await skillLibrary.load();
			await skillLibrary.load();

			expect(skillLibrary.skills).toHaveLength(2);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			expect(skillLibrary.skills).toHaveLength(0);
		});

		it('should set isLoading during load', async () => {
			const mockSkills = [{ id: 1, name: 'test' }];

			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			const loadPromise = skillLibrary.load();

			expect(skillLibrary.isLoading).toBe(true);

			resolveInvoke!(mockSkills);
			await loadPromise;

			expect(skillLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			expect(skillLibrary.error).toContain('Network error');
			expect(skillLibrary.isLoading).toBe(false);
		});
	});

	describe('getSkillById', () => {
		it('should return correct skill by ID', async () => {
			const mockSkills = [
				{ id: 1, name: 'skill-1' },
				{ id: 2, name: 'skill-2', skillType: 'skill' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			const skill = skillLibrary.getSkillById(2);
			expect(skill?.name).toBe('skill-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			const skill = skillLibrary.getSkillById(999);
			expect(skill).toBeUndefined();
		});
	});

	describe('filtering', () => {
		it('should filter skills by search query on name', async () => {
			const mockSkills = [
				{ id: 1, name: 'commit-helper', description: 'Git commits' },
				{ id: 2, name: 'review-code', description: 'Code review', skillType: 'skill' },
				{ id: 3, name: 'format-file', description: 'Format files' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			skillLibrary.setSearch('commit');

			expect(skillLibrary.filteredSkills).toHaveLength(1);
			expect(skillLibrary.filteredSkills[0].name).toBe('commit-helper');
		});

		it('should filter skills by description', async () => {
			const mockSkills = [
				{ id: 1, name: 'skill-1', description: 'Git helper' },
				{ id: 2, name: 'skill-2', description: 'Code review', skillType: 'skill' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			skillLibrary.setSearch('review');

			expect(skillLibrary.filteredSkills).toHaveLength(1);
			expect(skillLibrary.filteredSkills[0].name).toBe('skill-2');
		});

		it('should filter skills by tags', async () => {
			const mockSkills = [
				{ id: 1, name: 'skill-1', tags: ['git', 'version-control'] },
				{ id: 2, name: 'skill-2', tags: ['formatting'], skillType: 'skill' },
				{ id: 3, name: 'skill-3', tags: ['git', 'automation'] }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			skillLibrary.setSearch('git');

			expect(skillLibrary.filteredSkills).toHaveLength(2);
		});

		it('should be case-insensitive', async () => {
			const mockSkills = [
				{ id: 1, name: 'GitHelper', description: 'Git helper' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			skillLibrary.setSearch('GITHELPER');

			expect(skillLibrary.filteredSkills).toHaveLength(1);
		});

		it('should return all skills when search is empty', async () => {
			const mockSkills = [
				{ id: 1, name: 'skill-1' },
				{ id: 2, name: 'skill-2', skillType: 'skill' }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			skillLibrary.setSearch('');

			expect(skillLibrary.filteredSkills).toHaveLength(2);
		});
	});

	describe('CRUD operations', () => {
		it('should create a skill and add to list', async () => {
			const newSkill = { id: 3, name: 'new-skill', description: 'New' as const };

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // initial load
				.mockResolvedValueOnce(newSkill); // create

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			const result = await skillLibrary.create({
				name: 'new-skill',
				description: 'New',
				content: 'Content'
			});

			expect(result.id).toBe(3);
			expect(skillLibrary.skills).toHaveLength(1);
			expect(skillLibrary.skills[0].name).toBe('new-skill');
		});

		it('should update a skill in the list', async () => {
			const mockSkills = [{ id: 1, name: 'old-name' }];
			const updatedSkill = { id: 1, name: 'new-name' };

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockSkills)
				.mockResolvedValueOnce(updatedSkill);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			await skillLibrary.update(1, {
				name: 'new-name',
				description: '',
				content: ''
			});

			expect(skillLibrary.skills[0].name).toBe('new-name');
		});

		it('should delete a skill from the list', async () => {
			const mockSkills = [
				{ id: 1, name: 'skill-1' },
				{ id: 2, name: 'skill-2', skillType: 'skill' }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockSkills)
				.mockResolvedValueOnce(undefined);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.load();

			await skillLibrary.delete(1);

			expect(skillLibrary.skills).toHaveLength(1);
			expect(skillLibrary.skills[0].id).toBe(2);
		});
	});

	describe('global skills', () => {
		it('should load global skills', async () => {
			const mockGlobalSkills = [
				{ id: 1, skill_id: 1, is_enabled: true, skill: { id: 1, name: 'global-skill' } }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.loadGlobalSkills();

			expect(skillLibrary.globalSkills).toHaveLength(1);
		});

		it('should add global skill', async () => {
			const mockGlobalSkills = [
				{ id: 1, skill_id: 1, is_enabled: true, skill: { id: 1, name: 'test' } }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // add_global_skill
				.mockResolvedValueOnce(mockGlobalSkills); // loadGlobalSkills

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.addGlobalSkill(1);

			expect(invoke).toHaveBeenCalledWith('add_global_skill', { skillId: 1 });
		});

		it('should remove global skill', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove_global_skill
				.mockResolvedValueOnce([]); // loadGlobalSkills

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.removeGlobalSkill(1);

			expect(invoke).toHaveBeenCalledWith('remove_global_skill', { skillId: 1 });
		});

		it('should toggle global skill', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle_global_skill
				.mockResolvedValueOnce([]); // loadGlobalSkills

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.toggleGlobalSkill(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_skill', { id: 1, enabled: false });
		});
	});

	describe('project skills', () => {
		it('should assign skill to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_skill_to_project', { projectId: 1, skillId: 2 });
		});

		it('should remove skill from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_skill_from_project', { projectId: 1, skillId: 2 });
		});

		it('should toggle project skill', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			await skillLibrary.toggleProjectSkill(5, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_skill', { assignmentId: 5, enabled: true });
		});

		it('should get project skills', async () => {
			const mockProjectSkills = [
				{ id: 1, skill_id: 1, is_enabled: true, skill: { id: 1, name: 'test' } }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjectSkills);

			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');
			const result = await skillLibrary.getProjectSkills(1);

			expect(result).toHaveLength(1);
			expect(invoke).toHaveBeenCalledWith('get_project_skills', { projectId: 1 });
		});
	});
});
