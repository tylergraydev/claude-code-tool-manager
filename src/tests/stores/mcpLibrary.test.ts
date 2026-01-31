import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Mcp, CreateMcpRequest } from '$lib/types';

describe('MCP Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockMcp = (overrides: Partial<Mcp> = {}): Mcp => ({
		id: 1,
		name: 'test-mcp',
		type: 'stdio',
		command: '/usr/bin/test',
		args: [],
		env: {},
		source: 'user',
		createdAt: '2024-01-01T00:00:00Z',
		updatedAt: '2024-01-01T00:00:00Z',
		...overrides
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');

			expect(mcpLibrary.mcps).toEqual([]);
			expect(mcpLibrary.isLoading).toBe(false);
			expect(mcpLibrary.error).toBeNull();
			expect(mcpLibrary.searchQuery).toBe('');
			expect(mcpLibrary.selectedType).toBe('all');
		});
	});

	describe('load', () => {
		it('should load MCPs without duplicates', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', type: 'stdio' }),
				createMockMcp({ id: 2, name: 'mcp-2', type: 'http' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			expect(mcpLibrary.mcps).toHaveLength(2);
			expect(mcpLibrary.mcps[0].name).toBe('mcp-1');
		});

		it('should not create duplicates on multiple loads', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', type: 'stdio' }),
				createMockMcp({ id: 2, name: 'mcp-2', type: 'http' })
			];

			vi.mocked(invoke).mockResolvedValue(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');

			await mcpLibrary.load();
			await mcpLibrary.load();
			await mcpLibrary.load();

			expect(mcpLibrary.mcps).toHaveLength(2);
		});

		it('should handle empty response', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			expect(mcpLibrary.mcps).toHaveLength(0);
		});

		it('should handle load errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load failed'));

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			expect(mcpLibrary.error).toBe('Error: Load failed');
			expect(mcpLibrary.isLoading).toBe(false);
		});

		it('should set isLoading during load', async () => {
			let resolvePromise: (value: Mcp[]) => void;
			const pendingPromise = new Promise<Mcp[]>((resolve) => {
				resolvePromise = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(pendingPromise);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			const loadPromise = mcpLibrary.load();

			expect(mcpLibrary.isLoading).toBe(true);

			resolvePromise!([]);
			await loadPromise;

			expect(mcpLibrary.isLoading).toBe(false);
		});
	});

	describe('create', () => {
		it('should create MCP and add to state', async () => {
			const newMcp = createMockMcp({ id: 3, name: 'new-mcp' });
			vi.mocked(invoke).mockResolvedValueOnce(newMcp);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			const request: CreateMcpRequest = {
				name: 'new-mcp',
				type: 'stdio',
				command: '/usr/bin/test',
				args: [],
				env: {}
			};

			const result = await mcpLibrary.create(request);

			expect(invoke).toHaveBeenCalledWith('create_mcp', { mcp: request });
			expect(result).toEqual(newMcp);
			expect(mcpLibrary.mcps).toContainEqual(newMcp);
		});
	});

	describe('update', () => {
		it('should update MCP in state', async () => {
			const existingMcp = createMockMcp({ id: 1, name: 'old-name' });
			const updatedMcp = createMockMcp({ id: 1, name: 'new-name' });

			vi.mocked(invoke)
				.mockResolvedValueOnce([existingMcp])
				.mockResolvedValueOnce(updatedMcp);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			const request: CreateMcpRequest = {
				name: 'new-name',
				type: 'stdio',
				command: '/usr/bin/test',
				args: [],
				env: {}
			};

			const result = await mcpLibrary.update(1, request);

			expect(invoke).toHaveBeenCalledWith('update_mcp', { id: 1, mcp: request });
			expect(result).toEqual(updatedMcp);
			expect(mcpLibrary.mcps[0].name).toBe('new-name');
		});
	});

	describe('delete', () => {
		it('should remove MCP from state', async () => {
			const mcp1 = createMockMcp({ id: 1, name: 'mcp-1' });
			const mcp2 = createMockMcp({ id: 2, name: 'mcp-2' });

			vi.mocked(invoke)
				.mockResolvedValueOnce([mcp1, mcp2])
				.mockResolvedValueOnce(undefined);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();
			expect(mcpLibrary.mcps).toHaveLength(2);

			await mcpLibrary.delete(1);

			expect(invoke).toHaveBeenCalledWith('delete_mcp', { id: 1 });
			expect(mcpLibrary.mcps).toHaveLength(1);
			expect(mcpLibrary.mcps[0].id).toBe(2);
		});
	});

	describe('duplicate', () => {
		it('should duplicate MCP and add to state', async () => {
			const existingMcp = createMockMcp({ id: 1, name: 'original' });
			const duplicatedMcp = createMockMcp({ id: 2, name: 'original (copy)' });

			vi.mocked(invoke)
				.mockResolvedValueOnce([existingMcp])
				.mockResolvedValueOnce(duplicatedMcp);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			const result = await mcpLibrary.duplicate(1);

			expect(invoke).toHaveBeenCalledWith('duplicate_mcp', { id: 1 });
			expect(result).toEqual(duplicatedMcp);
			expect(mcpLibrary.mcps).toHaveLength(2);
		});
	});

	describe('toggleGlobal', () => {
		it('should toggle global enabled status', async () => {
			const mcp = createMockMcp({ id: 1, isEnabledGlobal: false });

			vi.mocked(invoke)
				.mockResolvedValueOnce([mcp])
				.mockResolvedValueOnce(undefined);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			await mcpLibrary.toggleGlobal(1, true);

			expect(invoke).toHaveBeenCalledWith('toggle_global_mcp', { id: 1, enabled: true });
			expect(mcpLibrary.mcps[0].isEnabledGlobal).toBe(true);
		});
	});

	describe('getMcpById', () => {
		it('should return correct MCP by ID', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1' }),
				createMockMcp({ id: 2, name: 'mcp-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			const mcp = mcpLibrary.getMcpById(2);
			expect(mcp?.name).toBe('mcp-2');
		});

		it('should return undefined for non-existent ID', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			const mcp = mcpLibrary.getMcpById(999);
			expect(mcp).toBeUndefined();
		});
	});

	describe('setSearch', () => {
		it('should set search query', async () => {
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			mcpLibrary.setSearch('test query');
			expect(mcpLibrary.searchQuery).toBe('test query');
		});
	});

	describe('setTypeFilter', () => {
		it('should set type filter', async () => {
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			mcpLibrary.setTypeFilter('stdio');
			expect(mcpLibrary.selectedType).toBe('stdio');
		});

		it('should set type filter to sse', async () => {
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			mcpLibrary.setTypeFilter('sse');
			expect(mcpLibrary.selectedType).toBe('sse');
		});

		it('should set type filter to http', async () => {
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			mcpLibrary.setTypeFilter('http');
			expect(mcpLibrary.selectedType).toBe('http');
		});

		it('should reset type filter to all', async () => {
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			mcpLibrary.setTypeFilter('stdio');
			mcpLibrary.setTypeFilter('all');
			expect(mcpLibrary.selectedType).toBe('all');
		});
	});

	describe('filteredMcps', () => {
		it('should filter MCPs by search query in name', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'angular-cli', description: 'Angular CLI tool' }),
				createMockMcp({ id: 2, name: 'github-mcp', description: 'GitHub integration' }),
				createMockMcp({ id: 3, name: 'huggingface', description: 'HuggingFace connector' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('github');

			expect(mcpLibrary.filteredMcps).toHaveLength(1);
			expect(mcpLibrary.filteredMcps[0].name).toBe('github-mcp');
		});

		it('should filter MCPs by search query in description', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', description: 'Database connector' }),
				createMockMcp({ id: 2, name: 'mcp-2', description: 'File system helper' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('database');

			expect(mcpLibrary.filteredMcps).toHaveLength(1);
			expect(mcpLibrary.filteredMcps[0].name).toBe('mcp-1');
		});

		it('should filter MCPs by search query in tags', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', tags: ['database', 'sql'] }),
				createMockMcp({ id: 2, name: 'mcp-2', tags: ['api', 'rest'] })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('sql');

			expect(mcpLibrary.filteredMcps).toHaveLength(1);
			expect(mcpLibrary.filteredMcps[0].name).toBe('mcp-1');
		});

		it('should filter case-insensitively', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'GitHub-MCP', description: 'GitHub integration' }),
				createMockMcp({ id: 2, name: 'Other-MCP', description: 'Other stuff' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('GITHUB');

			expect(mcpLibrary.filteredMcps).toHaveLength(1);
			expect(mcpLibrary.filteredMcps[0].name).toBe('GitHub-MCP');
		});

		it('should filter MCPs by type', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', type: 'stdio' }),
				createMockMcp({ id: 2, name: 'mcp-2', type: 'http' }),
				createMockMcp({ id: 3, name: 'mcp-3', type: 'stdio' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setTypeFilter('stdio');

			expect(mcpLibrary.filteredMcps).toHaveLength(2);
			expect(mcpLibrary.filteredMcps.every((m) => m.type === 'stdio')).toBe(true);
		});

		it('should filter by both search and type', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'test-stdio', type: 'stdio' }),
				createMockMcp({ id: 2, name: 'test-http', type: 'http' }),
				createMockMcp({ id: 3, name: 'other-stdio', type: 'stdio' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('test');
			mcpLibrary.setTypeFilter('stdio');

			expect(mcpLibrary.filteredMcps).toHaveLength(1);
			expect(mcpLibrary.filteredMcps[0].name).toBe('test-stdio');
		});

		it('should return all MCPs when no filters applied', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1' }),
				createMockMcp({ id: 2, name: 'mcp-2' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			expect(mcpLibrary.filteredMcps).toHaveLength(2);
		});

		it('should handle MCPs without description', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', description: undefined }),
				createMockMcp({ id: 2, name: 'mcp-2', description: 'Has description' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('mcp');

			// Both should match by name
			expect(mcpLibrary.filteredMcps).toHaveLength(2);
		});

		it('should handle MCPs without tags', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, name: 'mcp-1', tags: undefined }),
				createMockMcp({ id: 2, name: 'mcp-2', tags: ['tag1'] })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			mcpLibrary.setSearch('tag1');

			expect(mcpLibrary.filteredMcps).toHaveLength(1);
			expect(mcpLibrary.filteredMcps[0].id).toBe(2);
		});
	});

	describe('mcpCount', () => {
		it('should count MCPs by type', async () => {
			const mockMcps = [
				createMockMcp({ id: 1, type: 'stdio' }),
				createMockMcp({ id: 2, type: 'stdio' }),
				createMockMcp({ id: 3, type: 'http' }),
				createMockMcp({ id: 4, type: 'sse' })
			];

			vi.mocked(invoke).mockResolvedValueOnce(mockMcps);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			expect(mcpLibrary.mcpCount.total).toBe(4);
			expect(mcpLibrary.mcpCount.stdio).toBe(2);
			expect(mcpLibrary.mcpCount.http).toBe(1);
			expect(mcpLibrary.mcpCount.sse).toBe(1);
		});

		it('should handle empty MCP list', async () => {
			vi.mocked(invoke).mockResolvedValueOnce([]);

			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');
			await mcpLibrary.load();

			expect(mcpLibrary.mcpCount.total).toBe(0);
			expect(mcpLibrary.mcpCount.stdio).toBe(0);
			expect(mcpLibrary.mcpCount.http).toBe(0);
			expect(mcpLibrary.mcpCount.sse).toBe(0);
		});
	});
});
