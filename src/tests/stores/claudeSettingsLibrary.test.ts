import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Claude Settings Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load settings successfully', async () => {
			const mock = { user: { apiKey: 'key' }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mock);
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			expect(claudeSettingsLibrary.settings).toEqual(mock);
			expect(claudeSettingsLibrary.isLoading).toBe(false);
		});

		it('should handle load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			expect(claudeSettingsLibrary.error).toBe('Error: fail');
		});
	});

	describe('currentScopeSettings', () => {
		it('should return user settings by default', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { theme: 'dark' }, project: { theme: 'light' }, local: null });
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			expect(claudeSettingsLibrary.currentScopeSettings).toEqual({ theme: 'dark' });
		});

		it('should return project settings', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: {}, project: { theme: 'light' }, local: null });
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			claudeSettingsLibrary.setScope('project');
			expect(claudeSettingsLibrary.currentScopeSettings).toEqual({ theme: 'light' });
		});

		it('should return local settings', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: {}, project: null, local: { theme: 'system' } });
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			claudeSettingsLibrary.setScope('local');
			expect(claudeSettingsLibrary.currentScopeSettings).toEqual({ theme: 'system' });
		});

		it('should return null when no settings loaded', async () => {
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			expect(claudeSettingsLibrary.currentScopeSettings).toBeNull();
		});

		it('should return null for missing scope', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: {}, project: null, local: null });
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			claudeSettingsLibrary.setScope('project');
			expect(claudeSettingsLibrary.currentScopeSettings).toBeNull();
		});
	});

	describe('save', () => {
		it('should save and reload', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: {}, project: null, local: null });
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // save
			vi.mocked(invoke).mockResolvedValueOnce({ user: { theme: 'new' }, project: null, local: null }); // reload

			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			await claudeSettingsLibrary.save({ theme: 'new' } as any);

			expect(invoke).toHaveBeenCalledWith('save_claude_settings', {
				scope: 'user', projectPath: null, settings: { theme: 'new' }
			});
		});

		it('should throw on save error', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: {}, project: null, local: null });
			vi.mocked(invoke).mockRejectedValueOnce(new Error('save fail'));

			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			await claudeSettingsLibrary.load();
			await expect(claudeSettingsLibrary.save({} as any)).rejects.toThrow('save fail');
		});
	});

	describe('setProjectPath', () => {
		it('should reset scope to user when path cleared from non-user scope', async () => {
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			claudeSettingsLibrary.setScope('project');
			claudeSettingsLibrary.setProjectPath(null);
			expect(claudeSettingsLibrary.selectedScope).toBe('user');
		});

		it('should keep user scope when path cleared from user scope', async () => {
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			claudeSettingsLibrary.setProjectPath(null);
			expect(claudeSettingsLibrary.selectedScope).toBe('user');
		});

		it('should set project path', async () => {
			const { claudeSettingsLibrary } = await import('$lib/stores/claudeSettingsLibrary.svelte');
			claudeSettingsLibrary.setProjectPath('/my/project');
			expect(claudeSettingsLibrary.projectPath).toBe('/my/project');
		});
	});
});
