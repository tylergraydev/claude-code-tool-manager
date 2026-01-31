import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { ClaudeJsonMcp, ClaudeJsonProject } from '$lib/stores/claudeJson.svelte';

describe('Claude JSON Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockMcp = (overrides: Partial<ClaudeJsonMcp> = {}): ClaudeJsonMcp => ({
		name: 'test-mcp',
		type: 'stdio',
		command: '/usr/bin/test',
		args: ['--help'],
		isEnabled: true,
		...overrides
	});

	const createMockProject = (overrides: Partial<ClaudeJsonProject> = {}): ClaudeJsonProject => ({
		path: '/path/to/project',
		mcps: [],
		...overrides
	});

	describe('loadAll', () => {
		it('should load MCPs and projects successfully', async () => {
			const mockMcps = [
				createMockMcp({ name: 'mcp-1' }),
				createMockMcp({ name: 'mcp-2', projectPath: '/project/path' })
			];
			const mockProjects = [createMockProject({ path: '/project/path' })];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce(mockProjects);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(invoke).toHaveBeenCalledWith('get_claude_json_mcps');
			expect(invoke).toHaveBeenCalledWith('get_claude_json_projects');
			expect(claudeJson.mcps).toHaveLength(2);
			expect(claudeJson.projects).toHaveLength(1);
			expect(claudeJson.isLoading).toBe(false);
			expect(claudeJson.error).toBeNull();
		});

		it('should set isLoading during load', async () => {
			let resolveMcps: (value: ClaudeJsonMcp[]) => void;
			let resolveProjects: (value: ClaudeJsonProject[]) => void;
			const mcpsPromise = new Promise<ClaudeJsonMcp[]>((resolve) => {
				resolveMcps = resolve;
			});
			const projectsPromise = new Promise<ClaudeJsonProject[]>((resolve) => {
				resolveProjects = resolve;
			});

			vi.mocked(invoke)
				.mockReturnValueOnce(mcpsPromise)
				.mockReturnValueOnce(projectsPromise);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			const loadPromise = claudeJson.loadAll();

			expect(claudeJson.isLoading).toBe(true);

			resolveMcps!([]);
			resolveProjects!([]);
			await loadPromise;

			expect(claudeJson.isLoading).toBe(false);
		});

		it('should handle errors during load', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.error).toBe('Error: Failed to load');
			expect(claudeJson.isLoading).toBe(false);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce([]);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcps).toHaveLength(0);
			expect(claudeJson.projects).toHaveLength(0);
		});
	});

	describe('globalMcps', () => {
		it('should return MCPs without projectPath', async () => {
			const mockMcps = [
				createMockMcp({ name: 'global-1' }),
				createMockMcp({ name: 'project-1', projectPath: '/path/1' }),
				createMockMcp({ name: 'global-2' }),
				createMockMcp({ name: 'project-2', projectPath: '/path/2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([]);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.globalMcps).toHaveLength(2);
			expect(claudeJson.globalMcps.every(m => !m.projectPath)).toBe(true);
		});
	});

	describe('mcpsByProject', () => {
		it('should group MCPs by project path', async () => {
			const mockMcps = [
				createMockMcp({ name: 'mcp-1', projectPath: '/project/a' }),
				createMockMcp({ name: 'mcp-2', projectPath: '/project/a' }),
				createMockMcp({ name: 'mcp-3', projectPath: '/project/b' }),
				createMockMcp({ name: 'global-1' }) // No project path
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([]);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			const byProject = claudeJson.mcpsByProject;
			expect(byProject.get('/project/a')).toHaveLength(2);
			expect(byProject.get('/project/b')).toHaveLength(1);
			expect(byProject.has('/project/c')).toBe(false);
		});

		it('should not include global MCPs in project groups', async () => {
			const mockMcps = [
				createMockMcp({ name: 'global-1' }),
				createMockMcp({ name: 'global-2' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([]);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcpsByProject.size).toBe(0);
		});
	});

	describe('toggleMcp', () => {
		it('should toggle MCP enabled state', async () => {
			const mockMcps = [
				createMockMcp({ name: 'test-mcp', projectPath: '/project', isEnabled: true })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();
			await claudeJson.toggleMcp('/project', 'test-mcp', false);

			expect(invoke).toHaveBeenCalledWith('toggle_mcp_in_claude_json', {
				projectPath: '/project',
				mcpName: 'test-mcp',
				enabled: false
			});
			expect(claudeJson.mcps[0].isEnabled).toBe(false);
		});

		it('should not modify non-matching MCPs', async () => {
			const mockMcps = [
				createMockMcp({ name: 'mcp-1', projectPath: '/project', isEnabled: true }),
				createMockMcp({ name: 'mcp-2', projectPath: '/project', isEnabled: true })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();
			await claudeJson.toggleMcp('/project', 'mcp-1', false);

			expect(claudeJson.mcps.find(m => m.name === 'mcp-2')?.isEnabled).toBe(true);
		});

		it('should handle errors and rethrow', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce([])
				.mockRejectedValueOnce(new Error('Toggle failed'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await expect(claudeJson.toggleMcp('/project', 'mcp', true)).rejects.toThrow('Toggle failed');
		});
	});

	describe('removeMcpFromProject', () => {
		it('should remove MCP from project', async () => {
			const mockMcps = [
				createMockMcp({ name: 'mcp-1', projectPath: '/project' }),
				createMockMcp({ name: 'mcp-2', projectPath: '/project' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcps).toHaveLength(2);

			await claudeJson.removeMcpFromProject('/project', 'mcp-1');

			expect(invoke).toHaveBeenCalledWith('remove_mcp_from_claude_json', {
				projectPath: '/project',
				mcpName: 'mcp-1'
			});
			expect(claudeJson.mcps).toHaveLength(1);
			expect(claudeJson.mcps[0].name).toBe('mcp-2');
		});

		it('should not remove MCPs from different projects', async () => {
			const mockMcps = [
				createMockMcp({ name: 'mcp-1', projectPath: '/project-a' }),
				createMockMcp({ name: 'mcp-1', projectPath: '/project-b' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();
			await claudeJson.removeMcpFromProject('/project-a', 'mcp-1');

			expect(claudeJson.mcps).toHaveLength(1);
			expect(claudeJson.mcps[0].projectPath).toBe('/project-b');
		});

		it('should handle errors and rethrow', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce([])
				.mockRejectedValueOnce(new Error('Remove failed'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await expect(claudeJson.removeMcpFromProject('/project', 'mcp')).rejects.toThrow('Remove failed');
		});
	});

	describe('removeGlobalMcp', () => {
		it('should remove global MCP', async () => {
			const mockMcps = [
				createMockMcp({ name: 'global-mcp' }),
				createMockMcp({ name: 'project-mcp', projectPath: '/project' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcps).toHaveLength(2);

			await claudeJson.removeGlobalMcp('global-mcp');

			expect(invoke).toHaveBeenCalledWith('remove_global_mcp_from_claude_json', {
				mcpName: 'global-mcp'
			});
			expect(claudeJson.mcps).toHaveLength(1);
			expect(claudeJson.mcps[0].name).toBe('project-mcp');
		});

		it('should not remove project MCPs with same name', async () => {
			const mockMcps = [
				createMockMcp({ name: 'same-name' }),
				createMockMcp({ name: 'same-name', projectPath: '/project' })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();
			await claudeJson.removeGlobalMcp('same-name');

			expect(claudeJson.mcps).toHaveLength(1);
			expect(claudeJson.mcps[0].projectPath).toBe('/project');
		});

		it('should handle errors and rethrow', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce([])
				.mockRejectedValueOnce(new Error('Remove failed'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await expect(claudeJson.removeGlobalMcp('mcp')).rejects.toThrow('Remove failed');
		});
	});
});
