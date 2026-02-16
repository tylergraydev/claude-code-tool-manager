import { invoke } from '@tauri-apps/api/core';
import type {
	AllPermissions,
	ScopedPermissions,
	PermissionTemplate,
	PermissionScope,
	PermissionCategory
} from '$lib/types';

class PermissionLibraryState {
	permissions = $state<AllPermissions | null>(null);
	templates = $state<PermissionTemplate[]>([]);
	isLoading = $state(false);
	error = $state<string | null>(null);
	selectedScope = $state<PermissionScope>('user');
	projectPath = $state<string | null>(null);
	searchQuery = $state('');

	currentScopePermissions = $derived.by(() => {
		if (!this.permissions) return null;
		switch (this.selectedScope) {
			case 'user':
				return this.permissions.user;
			case 'project':
				return this.permissions.project ?? null;
			case 'local':
				return this.permissions.local ?? null;
		}
	});

	filteredRules = $derived.by(() => {
		const perms = this.currentScopePermissions;
		if (!perms) return { allow: [], deny: [], ask: [] };

		if (!this.searchQuery) {
			return { allow: perms.allow, deny: perms.deny, ask: perms.ask };
		}

		const query = this.searchQuery.toLowerCase();
		return {
			allow: perms.allow.filter((r) => r.toLowerCase().includes(query)),
			deny: perms.deny.filter((r) => r.toLowerCase().includes(query)),
			ask: perms.ask.filter((r) => r.toLowerCase().includes(query))
		};
	});

	mergedView = $derived.by(() => {
		if (!this.permissions) return [];
		const result: { rule: string; category: PermissionCategory; scope: string }[] = [];

		const addRules = (perms: ScopedPermissions) => {
			for (const rule of perms.deny) {
				result.push({ rule, category: 'deny', scope: perms.scope });
			}
			for (const rule of perms.ask) {
				result.push({ rule, category: 'ask', scope: perms.scope });
			}
			for (const rule of perms.allow) {
				result.push({ rule, category: 'allow', scope: perms.scope });
			}
		};

		addRules(this.permissions.user);
		if (this.permissions.project) addRules(this.permissions.project);
		if (this.permissions.local) addRules(this.permissions.local);

		return result;
	});

	async load() {
		console.log('[permissionLibrary] Loading permissions...');
		this.isLoading = true;
		this.error = null;
		try {
			this.permissions = await invoke<AllPermissions>('get_all_permissions', {
				projectPath: this.projectPath
			});
			console.log('[permissionLibrary] Loaded permissions');
		} catch (e) {
			this.error = String(e);
			console.error('[permissionLibrary] Failed to load permissions:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadTemplates() {
		console.log('[permissionLibrary] Loading templates...');
		try {
			this.templates = await invoke<PermissionTemplate[]>('get_permission_templates');
			console.log(`[permissionLibrary] Loaded ${this.templates.length} templates`);
		} catch (e) {
			console.error('[permissionLibrary] Failed to load templates:', e);
		}
	}

	async seedTemplates() {
		try {
			await invoke('seed_permission_templates');
			await this.loadTemplates();
		} catch (e) {
			console.error('[permissionLibrary] Failed to seed templates:', e);
		}
	}

	async addRule(category: PermissionCategory, rule: string) {
		console.log(`[permissionLibrary] Adding ${category} rule: ${rule}`);
		try {
			await invoke('add_permission_rule', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				category,
				rule
			});
			await this.load();
		} catch (e) {
			console.error('[permissionLibrary] Failed to add rule:', e);
			throw e;
		}
	}

	async removeRule(category: PermissionCategory, index: number) {
		console.log(`[permissionLibrary] Removing ${category} rule at index ${index}`);
		try {
			await invoke('remove_permission_rule', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				category,
				index
			});
			await this.load();
		} catch (e) {
			console.error('[permissionLibrary] Failed to remove rule:', e);
			throw e;
		}
	}

	async reorderRules(category: PermissionCategory, rules: string[]) {
		console.log(`[permissionLibrary] Reordering ${category} rules`);
		try {
			await invoke('reorder_permission_rules', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				category,
				rules
			});
			await this.load();
		} catch (e) {
			console.error('[permissionLibrary] Failed to reorder rules:', e);
			throw e;
		}
	}

	async setDefaultMode(mode: string | null) {
		console.log(`[permissionLibrary] Setting defaultMode=${mode}`);
		try {
			await invoke('set_default_mode', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				mode: mode || null
			});
			await this.load();
		} catch (e) {
			console.error('[permissionLibrary] Failed to set default mode:', e);
			throw e;
		}
	}

	async setAdditionalDirectories(directories: string[]) {
		console.log(`[permissionLibrary] Setting ${directories.length} additional directories`);
		try {
			await invoke('set_additional_directories', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				directories
			});
			await this.load();
		} catch (e) {
			console.error('[permissionLibrary] Failed to set directories:', e);
			throw e;
		}
	}

	async applyTemplate(template: PermissionTemplate) {
		console.log(`[permissionLibrary] Applying template: ${template.name}`);
		await this.addRule(template.category, template.rule);
	}

	setScope(scope: PermissionScope) {
		this.selectedScope = scope;
	}

	setProjectPath(path: string | null) {
		this.projectPath = path;
		// Reset to user scope if project path is cleared
		if (!path && this.selectedScope !== 'user') {
			this.selectedScope = 'user';
		}
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}
}

export const permissionLibrary = new PermissionLibraryState();
