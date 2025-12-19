import { invoke } from '@tauri-apps/api/core';
import type { Skill, CreateSkillRequest, GlobalSkill, ProjectSkill } from '$lib/types';

class SkillLibraryState {
	skills = $state<Skill[]>([]);
	globalSkills = $state<GlobalSkill[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');

	filteredSkills = $derived.by(() => {
		let result = this.skills;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(s) =>
					s.name.toLowerCase().includes(query) ||
					s.description?.toLowerCase().includes(query) ||
					s.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		return result;
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.skills = await invoke<Skill[]>('get_all_skills');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load skills:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadGlobalSkills() {
		try {
			this.globalSkills = await invoke<GlobalSkill[]>('get_global_skills');
		} catch (e) {
			console.error('Failed to load global skills:', e);
		}
	}

	async create(request: CreateSkillRequest): Promise<Skill> {
		const skill = await invoke<Skill>('create_skill', { skill: request });
		this.skills = [...this.skills, skill];
		return skill;
	}

	async update(id: number, request: CreateSkillRequest): Promise<Skill> {
		const skill = await invoke<Skill>('update_skill', { id, skill: request });
		this.skills = this.skills.map((s) => (s.id === id ? skill : s));
		return skill;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_skill', { id });
		this.skills = this.skills.filter((s) => s.id !== id);
	}

	async addGlobalSkill(skillId: number): Promise<void> {
		await invoke('add_global_skill', { skillId });
		await this.loadGlobalSkills();
	}

	async removeGlobalSkill(skillId: number): Promise<void> {
		await invoke('remove_global_skill', { skillId });
		await this.loadGlobalSkills();
	}

	async toggleGlobalSkill(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_skill', { id, enabled });
		await this.loadGlobalSkills();
	}

	async assignToProject(projectId: number, skillId: number): Promise<void> {
		await invoke('assign_skill_to_project', { projectId, skillId });
	}

	async removeFromProject(projectId: number, skillId: number): Promise<void> {
		await invoke('remove_skill_from_project', { projectId, skillId });
	}

	async toggleProjectSkill(assignmentId: number, enabled: boolean): Promise<void> {
		await invoke('toggle_project_skill', { assignmentId, enabled });
	}

	async getProjectSkills(projectId: number): Promise<ProjectSkill[]> {
		return await invoke<ProjectSkill[]>('get_project_skills', { projectId });
	}

	getSkillById(id: number): Skill | undefined {
		return this.skills.find((s) => s.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const skillLibrary = new SkillLibraryState();
