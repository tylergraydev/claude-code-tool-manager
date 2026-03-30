import { invoke } from '@tauri-apps/api/core';
import type { Rule, CreateRuleRequest, GlobalRule, ProjectRule } from '$lib/types';

class RuleLibraryState {
	rules = $state<Rule[]>([]);
	globalRules = $state<GlobalRule[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');

	filteredRules = $derived.by(() => {
		let result = this.rules;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(r) =>
					r.name.toLowerCase().includes(query) ||
					r.description?.toLowerCase().includes(query) ||
					r.tags?.some((t) => t.toLowerCase().includes(query)) ||
					r.paths?.some((p) => p.toLowerCase().includes(query))
			);
		}

		// Sort by favorites first, then by name
		return [...result].sort((a, b) => {
			if (a.isFavorite !== b.isFavorite) {
				return a.isFavorite ? -1 : 1;
			}
			return a.name.localeCompare(b.name);
		});
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.rules = await invoke<Rule[]>('get_all_rules');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load rules:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadGlobalRules() {
		try {
			this.globalRules = await invoke<GlobalRule[]>('get_global_rules');
		} catch (e) {
			console.error('Failed to load global rules:', e);
		}
	}

	async create(request: CreateRuleRequest): Promise<Rule> {
		const rule = await invoke<Rule>('create_rule', { rule: request });
		this.rules = [...this.rules, rule];
		return rule;
	}

	async update(id: number, request: CreateRuleRequest): Promise<Rule> {
		const rule = await invoke<Rule>('update_rule', { id, rule: request });
		this.rules = this.rules.map((r) => (r.id === id ? rule : r));
		return rule;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_rule', { id });
		this.rules = this.rules.filter((r) => r.id !== id);
	}

	async addGlobalRule(ruleId: number): Promise<void> {
		await invoke('add_global_rule', { ruleId });
		await this.loadGlobalRules();
	}

	async removeGlobalRule(ruleId: number): Promise<void> {
		await invoke('remove_global_rule', { ruleId });
		await this.loadGlobalRules();
	}

	async toggleGlobalRule(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_rule', { id, enabled });
		await this.loadGlobalRules();
	}

	async assignToProject(projectId: number, ruleId: number): Promise<void> {
		await invoke('assign_rule_to_project', { projectId, ruleId });
	}

	async removeFromProject(projectId: number, ruleId: number): Promise<void> {
		await invoke('remove_rule_from_project', { projectId, ruleId });
	}

	async toggleProjectRule(assignmentId: number, enabled: boolean): Promise<void> {
		await invoke('toggle_project_rule', { assignmentId, enabled });
	}

	async getProjectRules(projectId: number): Promise<ProjectRule[]> {
		return await invoke<ProjectRule[]>('get_project_rules', { projectId });
	}

	async getActiveRulesForPath(filePath: string): Promise<Rule[]> {
		return await invoke<Rule[]>('get_active_rules_for_path', { filePath });
	}

	getRuleById(id: number): Rule | undefined {
		return this.rules.find((r) => r.id === id);
	}

	updateRule(rule: Rule): void {
		this.rules = this.rules.map((r) => (r.id === rule.id ? rule : r));
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const ruleLibrary = new RuleLibraryState();
