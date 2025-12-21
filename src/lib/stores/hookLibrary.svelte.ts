import { invoke } from '@tauri-apps/api/core';
import type {
	Hook,
	CreateHookRequest,
	GlobalHook,
	ProjectHook,
	HookEventType,
	Project
} from '$lib/types';

export type HookViewMode = 'all' | 'byScope';

export interface ProjectWithHooks {
	project: Project;
	hooks: ProjectHook[];
}

class HookLibraryState {
	hooks = $state<Hook[]>([]);
	templates = $state<Hook[]>([]);
	globalHooks = $state<GlobalHook[]>([]);
	projectsWithHooks = $state<ProjectWithHooks[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	eventFilter = $state<HookEventType | ''>('');
	viewMode = $state<HookViewMode>('all');

	// Hooks that are not assigned anywhere (unassigned)
	unassignedHooks = $derived.by(() => {
		const globalHookIds = new Set(this.globalHooks.map((gh) => gh.hookId));
		const projectHookIds = new Set(
			this.projectsWithHooks.flatMap((p) => p.hooks.map((ph) => ph.hookId))
		);
		return this.filteredHooks.filter(
			(h) => !globalHookIds.has(h.id) && !projectHookIds.has(h.id)
		);
	});

	filteredHooks = $derived.by(() => {
		let result = this.hooks;

		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(h) =>
					h.name.toLowerCase().includes(query) ||
					h.description?.toLowerCase().includes(query) ||
					h.tags?.some((t) => t.toLowerCase().includes(query))
			);
		}

		if (this.eventFilter) {
			result = result.filter((h) => h.eventType === this.eventFilter);
		}

		return result;
	});

	// Hooks grouped by event type
	hooksByEventType = $derived.by(() => {
		const groups: Record<string, Hook[]> = {};
		for (const hook of this.filteredHooks) {
			if (!groups[hook.eventType]) {
				groups[hook.eventType] = [];
			}
			groups[hook.eventType].push(hook);
		}
		// Sort event types in a logical order
		const eventOrder = [
			'SessionStart',
			'UserPromptSubmit',
			'PreToolUse',
			'PostToolUse',
			'Notification',
			'Stop',
			'SubagentStop',
			'SessionEnd'
		];
		return eventOrder
			.filter((et) => groups[et]?.length > 0)
			.map((eventType) => ({ eventType, hooks: groups[eventType] }));
	});

	async load() {
		this.isLoading = true;
		this.error = null;
		try {
			this.hooks = await invoke<Hook[]>('get_all_hooks');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load hooks:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadTemplates() {
		try {
			this.templates = await invoke<Hook[]>('get_hook_templates');
		} catch (e) {
			console.error('Failed to load hook templates:', e);
		}
	}

	async seedTemplates() {
		try {
			await invoke('seed_hook_templates');
			await this.loadTemplates();
		} catch (e) {
			console.error('Failed to seed hook templates:', e);
		}
	}

	async loadGlobalHooks() {
		try {
			this.globalHooks = await invoke<GlobalHook[]>('get_global_hooks');
		} catch (e) {
			console.error('Failed to load global hooks:', e);
		}
	}

	async loadAllProjectHooks() {
		try {
			const projects = await invoke<Project[]>('get_all_projects');
			const projectsWithHooks: ProjectWithHooks[] = [];

			for (const project of projects) {
				const hooks = await invoke<ProjectHook[]>('get_project_hooks', {
					projectId: project.id
				});
				if (hooks.length > 0) {
					projectsWithHooks.push({ project, hooks });
				}
			}

			this.projectsWithHooks = projectsWithHooks;
		} catch (e) {
			console.error('Failed to load project hooks:', e);
		}
	}

	setViewMode(mode: HookViewMode) {
		this.viewMode = mode;
	}

	async create(request: CreateHookRequest): Promise<Hook> {
		const hook = await invoke<Hook>('create_hook', { hook: request });
		this.hooks = [...this.hooks, hook];
		return hook;
	}

	async createFromTemplate(templateId: number, name: string): Promise<Hook> {
		const hook = await invoke<Hook>('create_hook_from_template', { templateId, name });
		this.hooks = [...this.hooks, hook];
		return hook;
	}

	async update(id: number, request: CreateHookRequest): Promise<Hook> {
		const hook = await invoke<Hook>('update_hook', { id, hook: request });
		this.hooks = this.hooks.map((h) => (h.id === id ? hook : h));
		return hook;
	}

	async delete(id: number): Promise<void> {
		await invoke('delete_hook', { id });
		this.hooks = this.hooks.filter((h) => h.id !== id);
	}

	async addGlobalHook(hookId: number): Promise<void> {
		await invoke('add_global_hook', { hookId });
		await this.loadGlobalHooks();
	}

	async removeGlobalHook(hookId: number): Promise<void> {
		await invoke('remove_global_hook', { hookId });
		await this.loadGlobalHooks();
	}

	async toggleGlobalHook(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_global_hook', { id, enabled });
		await this.loadGlobalHooks();
	}

	async assignToProject(projectId: number, hookId: number): Promise<void> {
		await invoke('assign_hook_to_project', { projectId, hookId });
	}

	async removeFromProject(projectId: number, hookId: number): Promise<void> {
		await invoke('remove_hook_from_project', { projectId, hookId });
	}

	async toggleProjectHook(assignmentId: number, enabled: boolean): Promise<void> {
		await invoke('toggle_project_hook', { assignmentId, enabled });
	}

	async getProjectHooks(projectId: number): Promise<ProjectHook[]> {
		return await invoke<ProjectHook[]>('get_project_hooks', { projectId });
	}

	getHookById(id: number): Hook | undefined {
		return this.hooks.find((h) => h.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}

	setEventFilter(eventType: HookEventType | '') {
		this.eventFilter = eventType;
	}
}

export const hookLibrary = new HookLibraryState();
