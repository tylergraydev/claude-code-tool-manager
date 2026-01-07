import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Claude JSON Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	describe('loadAll', () => {
		it('should load all MCPs and projects successfully', async () => {
			const mockMcps = [
				{
					name: 'filesystem',
					type: 'stdio' as const,
					command: 'npx -y @modelcontextprotocol/server-filesystem',
					args: ['/path/to/dir'],
					isEnabled: true,
					projectPath: '/project/path'
				},
				{
					name: 'global-mcp',
					type: 'stdio' as const,
					command: 'npx -y mcp-server',
					args: [],
					isEnabled: true
				}
			];

			const mockProjects = [
				{
					path: '/project/path',
					mcps: [
						{
							name: 'filesystem',
							type: 'stdio' as const,
							command: 'npx -y @modelcontextprotocol/server-filesystem',
							args: ['/path/to/dir'],
							isEnabled: true,
							projectPath: '/project/path'
						}
					]
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce(mockProjects);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcps).toHaveLength(2);
			expect(claudeJson.projects).toHaveLength(1);
			expect(claudeJson.isLoading).toBe(false);
		});

		it('should handle errors gracefully', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load claude.json'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.error).toContain('Failed to load claude.json');
			expect(claudeJson.isLoading).toBe(false);
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			const loadPromise = claudeJson.loadAll();

			expect(claudeJson.isLoading).toBe(true);

			resolveInvoke!([]);
			await loadPromise;

			expect(claudeJson.isLoading).toBe(false);
		});
	});

	describe('derived properties', () => {
		it('should compute globalMcps correctly', async () => {
			const mockMcps = [
				{
					name: 'global-1',
					type: 'stdio' as const,
					command: 'npx mcp1',
					isEnabled: true
				},
				{
					name: 'project-mcp',
					type: 'stdio' as const,
					command: 'npx mcp2',
					projectPath: '/project',
					isEnabled: true
				},
				{
					name: 'global-2',
					type: 'stdio' as const,
					command: 'npx mcp3',
					isEnabled: true
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps).mockResolvedValueOnce([]);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.globalMcps).toHaveLength(2);
			expect(claudeJson.globalMcps.every((m) => !m.projectPath)).toBe(true);
		});

		it('should group MCPs by project', async () => {
			const mockMcps = [
				{
					name: 'mcp1',
					type: 'stdio' as const,
					command: 'npx mcp1',
					projectPath: '/project1',
					isEnabled: true
				},
				{
					name: 'mcp2',
					type: 'stdio' as const,
					command: 'npx mcp2',
					projectPath: '/project1',
					isEnabled: true
				},
				{
					name: 'mcp3',
					type: 'stdio' as const,
					command: 'npx mcp3',
					projectPath: '/project2',
					isEnabled: true
				}
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps).mockResolvedValueOnce([]);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			const project1Mcps = claudeJson.mcpsByProject.get('/project1');
			const project2Mcps = claudeJson.mcpsByProject.get('/project2');

			expect(project1Mcps).toHaveLength(2);
			expect(project2Mcps).toHaveLength(1);
			expect(project1Mcps![0].name).toBe('mcp1');
			expect(project1Mcps![1].name).toBe('mcp2');
			expect(project2Mcps![0].name).toBe('mcp3');
		});
	});

	describe('toggleMcp', () => {
		it('should toggle MCP enabled state', async () => {
			const mockMcps = [
				{
					name: 'test-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					projectPath: '/project',
					isEnabled: false
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined); // toggle_mcp_in_claude_json

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await claudeJson.toggleMcp('/project', 'test-mcp', true);

			expect(claudeJson.mcps[0].isEnabled).toBe(true);
			expect(invoke).toHaveBeenCalledWith('toggle_mcp_in_claude_json', {
				projectPath: '/project',
				mcpName: 'test-mcp',
				enabled: true
			});
		});

		it('should update local state after toggle', async () => {
			const mockMcps = [
				{
					name: 'test-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					projectPath: '/project',
					isEnabled: false
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await claudeJson.toggleMcp('/project', 'test-mcp', true);

			// Check the updated state after toggle completes
			const updatedMcp = claudeJson.mcps.find((m) => m.name === 'test-mcp');
			expect(updatedMcp?.isEnabled).toBe(true);
		});

		it('should throw error on failed toggle', async () => {
			const mockMcps = [
				{
					name: 'test-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					projectPath: '/project',
					isEnabled: false
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockRejectedValueOnce(new Error('Toggle failed'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await expect(claudeJson.toggleMcp('/project', 'test-mcp', true)).rejects.toThrow('Toggle failed');

			expect(claudeJson.mcps[0].isEnabled).toBe(false); // Should not change
		});
	});

	describe('removeMcpFromProject', () => {
		it('should remove MCP from project', async () => {
			const mockMcps = [
				{
					name: 'mcp1',
					type: 'stdio' as const,
					command: 'npx mcp1',
					projectPath: '/project',
					isEnabled: true
				},
				{
					name: 'mcp2',
					type: 'stdio' as const,
					command: 'npx mcp2',
					projectPath: '/project',
					isEnabled: true
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined); // remove_mcp_from_claude_json

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await claudeJson.removeMcpFromProject('/project', 'mcp1');

			expect(claudeJson.mcps).toHaveLength(1);
			expect(claudeJson.mcps[0].name).toBe('mcp2');
			expect(invoke).toHaveBeenCalledWith('remove_mcp_from_claude_json', {
				projectPath: '/project',
				mcpName: 'mcp1'
			});
		});

		it('should update local state after removal', async () => {
			const mockMcps = [
				{
					name: 'test-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					projectPath: '/project',
					isEnabled: true
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcps).toHaveLength(1);

			await claudeJson.removeMcpFromProject('/project', 'test-mcp');

			expect(claudeJson.mcps).toHaveLength(0);
		});

		it('should throw error on failed removal', async () => {
			const mockMcps = [
				{
					name: 'test-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					projectPath: '/project',
					isEnabled: true
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockRejectedValueOnce(new Error('Remove failed'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await expect(claudeJson.removeMcpFromProject('/project', 'test-mcp')).rejects.toThrow('Remove failed');

			expect(claudeJson.mcps).toHaveLength(1); // Should not change
		});
	});

	describe('removeGlobalMcp', () => {
		it('should remove global MCP', async () => {
			const mockMcps = [
				{
					name: 'global-1',
					type: 'stdio' as const,
					command: 'npx global1',
					isEnabled: true
				},
				{
					name: 'global-2',
					type: 'stdio' as const,
					command: 'npx global2',
					isEnabled: true
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined); // remove_global_mcp_from_claude_json

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await claudeJson.removeGlobalMcp('global-1');

			expect(claudeJson.mcps).toHaveLength(1);
			expect(claudeJson.mcps[0].name).toBe('global-2');
			expect(invoke).toHaveBeenCalledWith('remove_global_mcp_from_claude_json', { mcpName: 'global-1' });
		});

		it('should update local state after removal', async () => {
			const mockMcps = [
				{
					name: 'global-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					isEnabled: true
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce(undefined);

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			expect(claudeJson.mcps).toHaveLength(1);

			await claudeJson.removeGlobalMcp('global-mcp');

			expect(claudeJson.mcps).toHaveLength(0);
		});

		it('should throw error on failed removal', async () => {
			const mockMcps = [
				{
					name: 'global-mcp',
					type: 'stdio' as const,
					command: 'npx test',
					isEnabled: true
				}
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(mockMcps)
				.mockResolvedValueOnce([])
				.mockRejectedValueOnce(new Error('Remove failed'));

			const { claudeJson } = await import('$lib/stores/claudeJson.svelte');
			await claudeJson.loadAll();

			await expect(claudeJson.removeGlobalMcp('global-mcp')).rejects.toThrow('Remove failed');

			expect(claudeJson.mcps).toHaveLength(1); // Should not change
		});
	});
});
