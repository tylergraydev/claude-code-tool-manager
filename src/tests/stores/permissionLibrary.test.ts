import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Permission Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('load', () => {
		it('should load permissions successfully', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: ['Read'], deny: ['Bash'], ask: [] },
				project: null,
				local: null
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			expect(permissionLibrary.permissions).toEqual(mockPerms);
			expect(permissionLibrary.isLoading).toBe(false);
		});

		it('should handle load error', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			expect(permissionLibrary.error).toBe('Error: fail');
		});
	});

	describe('currentScopePermissions', () => {
		it('should return user permissions by default', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: ['Read'], deny: [], ask: [] },
				project: { scope: 'project', allow: ['Write'], deny: [], ask: [] },
				local: { scope: 'local', allow: ['Bash'], deny: [], ask: [] }
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			expect(permissionLibrary.currentScopePermissions?.allow).toEqual(['Read']);
		});

		it('should return project permissions when scope is project', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: [], deny: [], ask: [] },
				project: { scope: 'project', allow: ['Write'], deny: [], ask: [] },
				local: null
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			permissionLibrary.setScope('project');

			expect(permissionLibrary.currentScopePermissions?.allow).toEqual(['Write']);
		});

		it('should return local permissions when scope is local', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: [], deny: [], ask: [] },
				project: null,
				local: { scope: 'local', allow: ['Bash'], deny: [], ask: [] }
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			permissionLibrary.setScope('local');

			expect(permissionLibrary.currentScopePermissions?.allow).toEqual(['Bash']);
		});

		it('should return null when no permissions loaded', async () => {
			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			expect(permissionLibrary.currentScopePermissions).toBeNull();
		});

		it('should return null for project scope when project is null', async () => {
			const mockPerms = { user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			permissionLibrary.setScope('project');

			expect(permissionLibrary.currentScopePermissions).toBeNull();
		});
	});

	describe('filteredRules', () => {
		it('should return all rules when no search', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: ['Read', 'Write'], deny: ['Bash'], ask: ['Edit'] },
				project: null, local: null
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			expect(permissionLibrary.filteredRules.allow).toEqual(['Read', 'Write']);
			expect(permissionLibrary.filteredRules.deny).toEqual(['Bash']);
			expect(permissionLibrary.filteredRules.ask).toEqual(['Edit']);
		});

		it('should filter rules by search query', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: ['Read', 'Write'], deny: ['Bash'], ask: [] },
				project: null, local: null
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			permissionLibrary.setSearch('read');

			expect(permissionLibrary.filteredRules.allow).toEqual(['Read']);
			expect(permissionLibrary.filteredRules.deny).toEqual([]);
		});

		it('should return empty when no permissions', async () => {
			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			expect(permissionLibrary.filteredRules).toEqual({ allow: [], deny: [], ask: [] });
		});
	});

	describe('mergedView', () => {
		it('should merge rules from all scopes', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: ['Read'], deny: ['Bash'], ask: ['Edit'] },
				project: { scope: 'project', allow: ['Write'], deny: [], ask: [] },
				local: null
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			const merged = permissionLibrary.mergedView;
			expect(merged.length).toBe(4);
			// Order: deny, ask, allow for each scope
			expect(merged[0]).toEqual({ rule: 'Bash', category: 'deny', scope: 'user' });
			expect(merged[1]).toEqual({ rule: 'Edit', category: 'ask', scope: 'user' });
			expect(merged[2]).toEqual({ rule: 'Read', category: 'allow', scope: 'user' });
			expect(merged[3]).toEqual({ rule: 'Write', category: 'allow', scope: 'project' });
		});

		it('should return empty when no permissions', async () => {
			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			expect(permissionLibrary.mergedView).toEqual([]);
		});

		it('should include local scope when present', async () => {
			const mockPerms = {
				user: { scope: 'user', allow: [], deny: [], ask: [] },
				project: null,
				local: { scope: 'local', allow: ['Bash'], deny: [], ask: [] }
			};
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			expect(permissionLibrary.mergedView).toEqual([{ rule: 'Bash', category: 'allow', scope: 'local' }]);
		});
	});

	describe('addRule', () => {
		it('should add a rule and reload', async () => {
			const mockPerms = { user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms); // load
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // addRule
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms); // reload

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			await permissionLibrary.addRule('allow', 'Read');

			expect(invoke).toHaveBeenCalledWith('add_permission_rule', {
				scope: 'user', projectPath: null, category: 'allow', rule: 'Read'
			});
		});

		it('should throw on error', async () => {
			const mockPerms = { user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			await expect(permissionLibrary.addRule('allow', 'Read')).rejects.toThrow('fail');
		});
	});

	describe('removeRule', () => {
		it('should remove a rule and reload', async () => {
			const mockPerms = { user: { scope: 'user', allow: ['Read'], deny: [], ask: [] }, project: null, local: null };
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce(mockPerms);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			await permissionLibrary.removeRule('allow', 0);

			expect(invoke).toHaveBeenCalledWith('remove_permission_rule', {
				scope: 'user', projectPath: null, category: 'allow', index: 0
			});
		});

		it('should throw on error', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			await expect(permissionLibrary.removeRule('allow', 0)).rejects.toThrow('fail');
		});
	});

	describe('reorderRules', () => {
		it('should reorder and reload', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: ['A', 'B'], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: ['B', 'A'], deny: [], ask: [] }, project: null, local: null });

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			await permissionLibrary.reorderRules('allow', ['B', 'A']);

			expect(invoke).toHaveBeenCalledWith('reorder_permission_rules', {
				scope: 'user', projectPath: null, category: 'allow', rules: ['B', 'A']
			});
		});

		it('should throw on error', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			await expect(permissionLibrary.reorderRules('allow', [])).rejects.toThrow('fail');
		});
	});

	describe('setDefaultMode', () => {
		it('should set mode and reload', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			await permissionLibrary.setDefaultMode('plan');

			expect(invoke).toHaveBeenCalledWith('set_default_mode', {
				scope: 'user', projectPath: null, mode: 'plan'
			});
		});

		it('should pass null for empty string mode', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			await permissionLibrary.setDefaultMode('');

			expect(invoke).toHaveBeenCalledWith('set_default_mode', {
				scope: 'user', projectPath: null, mode: null
			});
		});

		it('should throw on error', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			await expect(permissionLibrary.setDefaultMode('plan')).rejects.toThrow('fail');
		});
	});

	describe('setAdditionalDirectories', () => {
		it('should set directories and reload', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockResolvedValueOnce(undefined);
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();
			await permissionLibrary.setAdditionalDirectories(['/path/a', '/path/b']);

			expect(invoke).toHaveBeenCalledWith('set_additional_directories', {
				scope: 'user', projectPath: null, directories: ['/path/a', '/path/b']
			});
		});

		it('should throw on error', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			await expect(permissionLibrary.setAdditionalDirectories([])).rejects.toThrow('fail');
		});
	});

	describe('loadTemplates', () => {
		it('should load templates', async () => {
			const templates = [{ name: 'Web Dev', category: 'allow', rule: 'Read' }];
			vi.mocked(invoke).mockResolvedValueOnce(templates);

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.loadTemplates();

			expect(permissionLibrary.templates).toEqual(templates);
		});

		it('should handle error gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.loadTemplates();
			// Should not throw
		});
	});

	describe('seedTemplates', () => {
		it('should seed and reload templates', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // seed
			vi.mocked(invoke).mockResolvedValueOnce([]); // loadTemplates

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.seedTemplates();

			expect(invoke).toHaveBeenCalledWith('seed_permission_templates');
		});

		it('should handle seed error gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('fail'));

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.seedTemplates();
			// Should not throw
		});
	});

	describe('applyTemplate', () => {
		it('should add rule from template', async () => {
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: [], deny: [], ask: [] }, project: null, local: null });
			vi.mocked(invoke).mockResolvedValueOnce(undefined); // addRule
			vi.mocked(invoke).mockResolvedValueOnce({ user: { scope: 'user', allow: ['Read'], deny: [], ask: [] }, project: null, local: null });

			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			await permissionLibrary.load();

			await permissionLibrary.applyTemplate({ name: 'Test', category: 'allow' as any, rule: 'Read' } as any);

			expect(invoke).toHaveBeenCalledWith('add_permission_rule', expect.objectContaining({ category: 'allow', rule: 'Read' }));
		});
	});

	describe('setProjectPath', () => {
		it('should set project path', async () => {
			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			permissionLibrary.setProjectPath('/my/project');
			expect(permissionLibrary.projectPath).toBe('/my/project');
		});

		it('should reset to user scope when path cleared', async () => {
			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			permissionLibrary.setScope('project');
			permissionLibrary.setProjectPath(null);
			expect(permissionLibrary.selectedScope).toBe('user');
		});

		it('should not reset scope when path cleared and already user', async () => {
			const { permissionLibrary } = await import('$lib/stores/permissionLibrary.svelte');
			permissionLibrary.setScope('user');
			permissionLibrary.setProjectPath(null);
			expect(permissionLibrary.selectedScope).toBe('user');
		});
	});
});
