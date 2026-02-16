import { invoke } from '@tauri-apps/api/core';
import type { AllClaudeSettings, ClaudeSettings, ClaudeSettingsScope } from '$lib/types';

class ClaudeSettingsLibraryState {
	settings = $state<AllClaudeSettings | null>(null);
	isLoading = $state(false);
	error = $state<string | null>(null);
	selectedScope = $state<ClaudeSettingsScope>('user');
	projectPath = $state<string | null>(null);

	currentScopeSettings = $derived.by(() => {
		if (!this.settings) return null;
		switch (this.selectedScope) {
			case 'user':
				return this.settings.user;
			case 'project':
				return this.settings.project ?? null;
			case 'local':
				return this.settings.local ?? null;
		}
	});

	async load() {
		console.log('[claudeSettingsLibrary] Loading settings...');
		this.isLoading = true;
		this.error = null;
		try {
			this.settings = await invoke<AllClaudeSettings>('get_all_claude_settings', {
				projectPath: this.projectPath
			});
			console.log('[claudeSettingsLibrary] Loaded settings');
		} catch (e) {
			this.error = String(e);
			console.error('[claudeSettingsLibrary] Failed to load settings:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async save(settings: ClaudeSettings) {
		console.log(`[claudeSettingsLibrary] Saving settings for scope=${this.selectedScope}`);
		try {
			await invoke<ClaudeSettings>('save_claude_settings', {
				scope: this.selectedScope,
				projectPath: this.projectPath,
				settings
			});
			await this.load();
		} catch (e) {
			console.error('[claudeSettingsLibrary] Failed to save settings:', e);
			throw e;
		}
	}

	setScope(scope: ClaudeSettingsScope) {
		this.selectedScope = scope;
	}

	setProjectPath(path: string | null) {
		this.projectPath = path;
		if (!path && this.selectedScope !== 'user') {
			this.selectedScope = 'user';
		}
	}
}

export const claudeSettingsLibrary = new ClaudeSettingsLibraryState();
