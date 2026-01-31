import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Hook, GlobalHook, ProjectHook, Project } from '$lib/types';

// Helper to create mock hooks
const createMockHook = (overrides: Partial<Hook> = {}): Hook => ({
	id: 1,
	name: 'test-hook',
	eventType: 'Stop',
	hookType: 'command',
	command: 'echo done',
	source: 'user',
	isTemplate: false,
	createdAt: '2024-01-01',
	updatedAt: '2024-01-01',
	...overrides
});

// Helper to create mock global hooks
const createMockGlobalHook = (overrides: Partial<GlobalHook> = {}): GlobalHook => ({
	id: 1,
	hookId: 1,
	isEnabled: true,
	hook: createMockHook(),
	...overrides
});

// Helper to create mock project hooks
const createMockProjectHook = (overrides: Partial<ProjectHook> = {}): ProjectHook => ({
	id: 1,
	hookId: 1,
	isEnabled: true,
	hook: createMockHook(),
	...overrides
});

// Helper to create mock projects
const createMockProject = (overrides: Partial<Project> = {}): Project => ({
	id: 1,
	name: 'test-project',
	path: '/path/to/project',
	assignedMcps: [],
	...overrides
});

describe('Hook Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');

			expect(hookLibrary.hooks).toEqual([]);
			expect(hookLibrary.templates).toEqual([]);
			expect(hookLibrary.globalHooks).toEqual([]);
			expect(hookLibrary.projectsWithHooks).toEqual([]);
			expect(hookLibrary.isLoading).toBe(false);
			expect(hookLibrary.error).toBeNull();
			expect(hookLibrary.searchQuery).toBe('');
			expect(hookLibrary.eventFilter).toBe('');
			expect(hookLibrary.viewMode).toBe('all');
		});
	});

	describe('load', () => {
		it('should load hooks', async () => {
			const mockHooks = [
				{
					id: 1,
					name: 'Test Hook 1',
					eventType: 'Stop',
					hookType: 'command',
					command: 'echo done',
					source: 'user',
					isTemplate: false,
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				},
				{
					id: 2,
					name: 'Test Hook 2',
					eventType: 'Notification',
					hookType: 'command',
					command: 'afplay /sound.aiff',
					source: 'user',
					isTemplate: false,
					createdAt: '2024-01-01',
					updatedAt: '2024-01-01'
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			expect(hookLibrary.hooks).toHaveLength(2);
			expect(hookLibrary.hooks[0].name).toBe('Test Hook 1');
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockHooks = [
				{
					id: 1,
					name: 'Hook 1',
					eventType: 'Stop',
					hookType: 'command',
					source: 'user',
					isTemplate: false
				}
			];

			vi.mocked(invoke).mockResolvedValue(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');

			await hookLibrary.load();
			await hookLibrary.load();
			await hookLibrary.load();

			expect(hookLibrary.hooks).toHaveLength(1);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			expect(hookLibrary.hooks).toHaveLength(0);
		});

		it('should set isLoading during load', async () => {
			const mockHooks = [
				{ id: 1, name: 'test', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
			];

			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			const loadPromise = hookLibrary.load();

			expect(hookLibrary.isLoading).toBe(true);

			resolveInvoke!(mockHooks);
			await loadPromise;

			expect(hookLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			expect(hookLibrary.error).toContain('Database error');
			expect(hookLibrary.isLoading).toBe(false);
		});
	});

	describe('getHookById', () => {
		it('should return correct hook by ID', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			const hook = hookLibrary.getHookById(2);
			expect(hook?.name).toBe('hook-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			const hook = hookLibrary.getHookById(999);
			expect(hook).toBeUndefined();
		});
	});

	describe('filtering', () => {
		it('should filter hooks by search query on name', async () => {
			const mockHooks = [
				{ id: 1, name: 'sound-notification', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'log-tool-use', eventType: 'PostToolUse', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 3, name: 'format-check', eventType: 'PreToolUse', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			hookLibrary.setSearch('sound');

			expect(hookLibrary.filteredHooks).toHaveLength(1);
			expect(hookLibrary.filteredHooks[0].name).toBe('sound-notification');
		});

		it('should filter hooks by description', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', description: 'Play notification sound', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', description: 'Log tool usage', eventType: 'PostToolUse', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			hookLibrary.setSearch('notification');

			expect(hookLibrary.filteredHooks).toHaveLength(1);
			expect(hookLibrary.filteredHooks[0].name).toBe('hook-1');
		});

		it('should filter hooks by tags', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', tags: ['sound', 'notification'], eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', tags: ['logging'], eventType: 'PostToolUse', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 3, name: 'hook-3', tags: ['sound', 'automation'], eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			hookLibrary.setSearch('sound');

			expect(hookLibrary.filteredHooks).toHaveLength(2);
		});

		it('should filter hooks by event type', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 3, name: 'hook-3', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			hookLibrary.setEventFilter('Stop');

			expect(hookLibrary.filteredHooks).toHaveLength(2);
			expect(hookLibrary.filteredHooks.every((h) => h.eventType === 'Stop')).toBe(true);
		});

		it('should be case-insensitive', async () => {
			const mockHooks = [
				{ id: 1, name: 'SoundNotification', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			hookLibrary.setSearch('SOUNDNOTIFICATION');

			expect(hookLibrary.filteredHooks).toHaveLength(1);
		});

		it('should return all hooks when search is empty', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			hookLibrary.setSearch('');

			expect(hookLibrary.filteredHooks).toHaveLength(2);
		});
	});

	describe('CRUD operations', () => {
		it('should create a hook and add to list', async () => {
			const newHook = {
				id: 3,
				name: 'new-hook',
				description: 'New Hook',
				eventType: 'Stop' as const,
				hookType: 'command' as const,
				command: 'echo done',
				source: 'user',
				isTemplate: false,
				createdAt: '2024-01-01',
				updatedAt: '2024-01-01'
			};

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // initial load
				.mockResolvedValueOnce(newHook); // create

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			const result = await hookLibrary.create({
				name: 'new-hook',
				description: 'New Hook',
				eventType: 'Stop',
				hookType: 'command',
				command: 'echo done'
			});

			expect(result.id).toBe(3);
			expect(hookLibrary.hooks).toHaveLength(1);
			expect(hookLibrary.hooks[0].name).toBe('new-hook');
		});

		it('should update a hook in the list', async () => {
			const mockHooks = [
				{ id: 1, name: 'old-name', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
			];
			const updatedHook = { id: 1, name: 'new-name', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false };

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockHooks)
				.mockResolvedValueOnce(updatedHook);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			await hookLibrary.update(1, {
				name: 'new-name',
				eventType: 'Stop',
				hookType: 'command',
				command: 'echo done'
			});

			expect(hookLibrary.hooks[0].name).toBe('new-name');
		});

		it('should only update the matching hook when multiple exist', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 3, name: 'hook-3', eventType: 'SessionStart', hookType: 'command', source: 'user', isTemplate: false }
			];
			const updatedHook = { id: 2, name: 'updated-hook-2', eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false };

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockHooks)
				.mockResolvedValueOnce(updatedHook);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			await hookLibrary.update(2, {
				name: 'updated-hook-2',
				eventType: 'Notification',
				hookType: 'command',
				command: 'echo done'
			});

			// Only hook 2 should be updated
			expect(hookLibrary.hooks[0].name).toBe('hook-1');
			expect(hookLibrary.hooks[1].name).toBe('updated-hook-2');
			expect(hookLibrary.hooks[2].name).toBe('hook-3');
			expect(hookLibrary.hooks).toHaveLength(3);
		});

		it('should delete a hook from the list', async () => {
			const mockHooks = [
				{ id: 1, name: 'hook-1', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false },
				{ id: 2, name: 'hook-2', eventType: 'Notification', hookType: 'command', source: 'user', isTemplate: false }
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockHooks)
				.mockResolvedValueOnce(undefined);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			await hookLibrary.delete(1);

			expect(hookLibrary.hooks).toHaveLength(1);
			expect(hookLibrary.hooks[0].id).toBe(2);
		});
	});

	describe('global hooks', () => {
		it('should load global hooks', async () => {
			const mockGlobalHooks = [
				{
					id: 1,
					hookId: 1,
					isEnabled: true,
					hook: { id: 1, name: 'global-hook', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockGlobalHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadGlobalHooks();

			expect(hookLibrary.globalHooks).toHaveLength(1);
		});

		it('should add global hook', async () => {
			const mockGlobalHooks = [
				{
					id: 1,
					hookId: 1,
					isEnabled: true,
					hook: { id: 1, name: 'test', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // add_global_hook
				.mockResolvedValueOnce(mockGlobalHooks); // loadGlobalHooks

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.addGlobalHook(1);

			expect(invoke).toHaveBeenCalledWith('add_global_hook', { hookId: 1 });
		});

		it('should remove global hook', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // remove_global_hook
				.mockResolvedValueOnce([]); // loadGlobalHooks

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.removeGlobalHook(1);

			expect(invoke).toHaveBeenCalledWith('remove_global_hook', { hookId: 1 });
		});

		it('should toggle global hook', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // toggle_global_hook
				.mockResolvedValueOnce([]); // loadGlobalHooks

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.toggleGlobalHook(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_global_hook', { id: 1, enabled: false });
		});
	});

	describe('project hooks', () => {
		it('should assign hook to project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.assignToProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('assign_hook_to_project', { projectId: 1, hookId: 2 });
		});

		it('should remove hook from project', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.removeFromProject(1, 2);

			expect(invoke).toHaveBeenCalledWith('remove_hook_from_project', { projectId: 1, hookId: 2 });
		});

		it('should toggle project hook', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(undefined);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.toggleProjectHook(5, true);

			expect(invoke).toHaveBeenCalledWith('toggle_project_hook', { assignmentId: 5, enabled: true });
		});

		it('should get project hooks', async () => {
			const mockProjectHooks = [
				{
					id: 1,
					hookId: 1,
					isEnabled: true,
					hook: { id: 1, name: 'test', eventType: 'Stop', hookType: 'command', source: 'user', isTemplate: false }
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockProjectHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			const result = await hookLibrary.getProjectHooks(1);

			expect(result).toHaveLength(1);
			expect(invoke).toHaveBeenCalledWith('get_project_hooks', { projectId: 1 });
		});
	});

	describe('templates', () => {
		it('should load templates', async () => {
			const mockTemplates = [
				{
					id: 1,
					name: 'Sound Notification',
					eventType: 'Stop',
					hookType: 'command',
					command: 'afplay /sound.aiff',
					source: 'template',
					isTemplate: true
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockTemplates);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadTemplates();

			expect(hookLibrary.templates).toHaveLength(1);
			expect(hookLibrary.templates[0].isTemplate).toBe(true);
		});

		it('should create hook from template', async () => {
			const newHook = {
				id: 2,
				name: 'My Sound Hook',
				eventType: 'Stop',
				hookType: 'command',
				command: 'afplay /sound.aiff',
				source: 'user',
				isTemplate: false
			};

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // initial load
				.mockResolvedValueOnce(newHook); // create from template

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			const result = await hookLibrary.createFromTemplate(1, 'My Sound Hook');

			expect(result.name).toBe('My Sound Hook');
			expect(invoke).toHaveBeenCalledWith('create_hook_from_template', { templateId: 1, name: 'My Sound Hook' });
			expect(hookLibrary.hooks).toHaveLength(1);
		});

		it('should seed templates', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined) // seed_hook_templates
				.mockResolvedValueOnce([]); // loadTemplates

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.seedTemplates();

			expect(invoke).toHaveBeenCalledWith('seed_hook_templates');
		});
	});

	describe('export functionality', () => {
		it('should export hooks to JSON', async () => {
			const mockJson = JSON.stringify([
				{ name: 'hook-1', eventType: 'Stop', hookType: 'command', command: 'echo done' }
			]);

			vi.mocked(invoke).mockResolvedValueOnce(mockJson);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			const result = await hookLibrary.exportToJson([1, 2]);

			expect(invoke).toHaveBeenCalledWith('export_hooks_to_json', { hookIds: [1, 2] });
			expect(result).toBe(mockJson);
		});

		it('should export hooks to clipboard', async () => {
			const mockJson = '{"hooks": []}';
			const mockWriteText = vi.fn().mockResolvedValue(undefined);
			Object.assign(navigator, {
				clipboard: { writeText: mockWriteText }
			});

			vi.mocked(invoke).mockResolvedValueOnce(mockJson);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.exportToClipboard([1]);

			expect(mockWriteText).toHaveBeenCalledWith(mockJson);
		});
	});

	describe('sound notification hooks', () => {
		it('should create sound notification hooks', async () => {
			const mockHooks = [
				{
					id: 1,
					name: 'Stop Sound',
					eventType: 'Stop',
					hookType: 'command',
					command: 'afplay /System/Library/Sounds/Glass.aiff',
					source: 'user',
					isTemplate: false
				},
				{
					id: 2,
					name: 'Notification Sound',
					eventType: 'Notification',
					hookType: 'command',
					command: 'afplay /System/Library/Sounds/Glass.aiff',
					source: 'user',
					isTemplate: false
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce([]) // initial load
				.mockResolvedValueOnce(mockHooks) // create_sound_notification_hooks
				.mockResolvedValueOnce([]); // loadGlobalHooks

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			const result = await hookLibrary.createSoundNotificationHooks(
				['Stop', 'Notification'],
				'/System/Library/Sounds/Glass.aiff',
				'shell'
			);

			expect(result).toHaveLength(2);
			expect(invoke).toHaveBeenCalledWith('create_sound_notification_hooks', {
				events: ['Stop', 'Notification'],
				soundPath: '/System/Library/Sounds/Glass.aiff',
				method: 'shell'
			});
			expect(hookLibrary.hooks).toHaveLength(2);
		});
	});

	describe('duplicate functionality', () => {
		it('should duplicate a hook', async () => {
			const originalHook = {
				id: 1,
				name: 'Original Hook',
				eventType: 'Stop',
				hookType: 'command',
				command: 'echo done',
				source: 'user',
				isTemplate: false
			};
			const duplicatedHook = {
				id: 2,
				name: 'Duplicated Hook',
				eventType: 'Stop',
				hookType: 'command',
				command: 'echo done',
				source: 'user',
				isTemplate: false
			};

			vi.mocked(invoke)
				.mockResolvedValueOnce([originalHook]) // initial load
				.mockResolvedValueOnce(duplicatedHook); // duplicate

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			const result = await hookLibrary.duplicate(1, 'Duplicated Hook');

			expect(result.name).toBe('Duplicated Hook');
			expect(invoke).toHaveBeenCalledWith('duplicate_hook', { id: 1, newName: 'Duplicated Hook' });
			expect(hookLibrary.hooks).toHaveLength(2);
		});
	});

	describe('view mode', () => {
		it('should set view mode', async () => {
			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');

			hookLibrary.setViewMode('byScope');
			expect(hookLibrary.viewMode).toBe('byScope');

			hookLibrary.setViewMode('all');
			expect(hookLibrary.viewMode).toBe('all');
		});
	});

	describe('loadAllProjectHooks', () => {
		it('should load hooks for all projects', async () => {
			const mockProjects = [
				createMockProject({ id: 1, name: 'project-1' }),
				createMockProject({ id: 2, name: 'project-2' })
			];
			const projectHooks1 = [createMockProjectHook({ id: 1, hookId: 1 })];
			const projectHooks2 = [createMockProjectHook({ id: 2, hookId: 2 })];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockProjects) // get_all_projects
				.mockResolvedValueOnce(projectHooks1) // get_project_hooks for project 1
				.mockResolvedValueOnce(projectHooks2); // get_project_hooks for project 2

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadAllProjectHooks();

			expect(hookLibrary.projectsWithHooks).toHaveLength(2);
			expect(hookLibrary.projectsWithHooks[0].project.id).toBe(1);
			expect(hookLibrary.projectsWithHooks[0].hooks).toHaveLength(1);
		});

		it('should only include projects with hooks', async () => {
			const mockProjects = [
				createMockProject({ id: 1, name: 'project-with-hooks' }),
				createMockProject({ id: 2, name: 'project-without-hooks' })
			];
			const projectHooks1 = [createMockProjectHook({ id: 1, hookId: 1 })];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockProjects) // get_all_projects
				.mockResolvedValueOnce(projectHooks1) // get_project_hooks for project 1 (has hooks)
				.mockResolvedValueOnce([]); // get_project_hooks for project 2 (no hooks)

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadAllProjectHooks();

			expect(hookLibrary.projectsWithHooks).toHaveLength(1);
			expect(hookLibrary.projectsWithHooks[0].project.name).toBe('project-with-hooks');
		});

		it('should handle errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load projects'));

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadAllProjectHooks();

			// Should not throw, just logs error
			expect(hookLibrary.projectsWithHooks).toEqual([]);
		});
	});

	describe('error handling', () => {
		it('should handle loadTemplates error silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load templates'));

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadTemplates();

			// Should not throw, just logs error
			expect(hookLibrary.templates).toEqual([]);
		});

		it('should handle seedTemplates error silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to seed templates'));

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.seedTemplates();

			// Should not throw, just logs error
			expect(hookLibrary.templates).toEqual([]);
		});

		it('should handle loadGlobalHooks error silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load global hooks'));

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.loadGlobalHooks();

			// Should not throw, just logs error
			expect(hookLibrary.globalHooks).toEqual([]);
		});
	});

	describe('hooksByEventType derived state', () => {
		it('should group hooks by event type', async () => {
			const mockHooks = [
				createMockHook({ id: 1, name: 'stop-hook-1', eventType: 'Stop' }),
				createMockHook({ id: 2, name: 'notification-hook', eventType: 'Notification' }),
				createMockHook({ id: 3, name: 'stop-hook-2', eventType: 'Stop' }),
				createMockHook({ id: 4, name: 'session-start', eventType: 'SessionStart' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			// hooksByEventType should be ordered by session lifecycle
			expect(hookLibrary.hooksByEventType.length).toBeGreaterThan(0);

			// SessionStart should come before Stop
			const sessionStartIndex = hookLibrary.hooksByEventType.findIndex(g => g.eventType === 'SessionStart');
			const stopIndex = hookLibrary.hooksByEventType.findIndex(g => g.eventType === 'Stop');
			expect(sessionStartIndex).toBeLessThan(stopIndex);

			// Stop group should have 2 hooks
			const stopGroup = hookLibrary.hooksByEventType.find(g => g.eventType === 'Stop');
			expect(stopGroup?.hooks).toHaveLength(2);
		});

		it('should only include event types with hooks', async () => {
			const mockHooks = [
				createMockHook({ id: 1, eventType: 'Stop' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();

			expect(hookLibrary.hooksByEventType).toHaveLength(1);
			expect(hookLibrary.hooksByEventType[0].eventType).toBe('Stop');
		});

		it('should respect filters', async () => {
			const mockHooks = [
				createMockHook({ id: 1, name: 'sound-stop', eventType: 'Stop' }),
				createMockHook({ id: 2, name: 'log-notification', eventType: 'Notification' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockHooks);

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();
			hookLibrary.setEventFilter('Stop');

			expect(hookLibrary.hooksByEventType).toHaveLength(1);
			expect(hookLibrary.hooksByEventType[0].eventType).toBe('Stop');
		});
	});

	describe('unassignedHooks derived state', () => {
		it('should return hooks not assigned globally or to any project', async () => {
			const mockHooks = [
				createMockHook({ id: 1, name: 'assigned-global' }),
				createMockHook({ id: 2, name: 'assigned-project' }),
				createMockHook({ id: 3, name: 'unassigned' })
			];
			const mockGlobalHooks = [createMockGlobalHook({ hookId: 1 })];
			const mockProjects = [createMockProject({ id: 1 })];
			const mockProjectHooks = [createMockProjectHook({ hookId: 2 })];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockHooks) // load
				.mockResolvedValueOnce(mockGlobalHooks) // loadGlobalHooks
				.mockResolvedValueOnce(mockProjects) // loadAllProjectHooks - get_all_projects
				.mockResolvedValueOnce(mockProjectHooks); // loadAllProjectHooks - get_project_hooks

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();
			await hookLibrary.loadGlobalHooks();
			await hookLibrary.loadAllProjectHooks();

			expect(hookLibrary.unassignedHooks).toHaveLength(1);
			expect(hookLibrary.unassignedHooks[0].name).toBe('unassigned');
		});

		it('should return all hooks when none are assigned', async () => {
			const mockHooks = [
				createMockHook({ id: 1, name: 'hook-1' }),
				createMockHook({ id: 2, name: 'hook-2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockHooks) // load
				.mockResolvedValueOnce([]) // loadGlobalHooks
				.mockResolvedValueOnce([]); // loadAllProjectHooks - get_all_projects

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();
			await hookLibrary.loadGlobalHooks();
			await hookLibrary.loadAllProjectHooks();

			expect(hookLibrary.unassignedHooks).toHaveLength(2);
		});

		it('should return empty array when all hooks are assigned', async () => {
			const mockHooks = [
				createMockHook({ id: 1, name: 'hook-1' }),
				createMockHook({ id: 2, name: 'hook-2' })
			];
			const mockGlobalHooks = [
				createMockGlobalHook({ hookId: 1 }),
				createMockGlobalHook({ hookId: 2 })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockHooks) // load
				.mockResolvedValueOnce(mockGlobalHooks) // loadGlobalHooks
				.mockResolvedValueOnce([]); // loadAllProjectHooks - get_all_projects

			const { hookLibrary } = await import('$lib/stores/hookLibrary.svelte');
			await hookLibrary.load();
			await hookLibrary.loadGlobalHooks();
			await hookLibrary.loadAllProjectHooks();

			expect(hookLibrary.unassignedHooks).toHaveLength(0);
		});
	});
});
