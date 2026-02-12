import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
	createMockRepo,
	createMockRepoItem,
	createMockSyncResult,
	createMockImportResult,
	createMockRegistryMcp,
	resetIdCounter
} from '../factories';

describe('Repo Library Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		resetIdCounter();
		vi.resetModules();
	});

	describe('loadRepos', () => {
		it('should load repos', async () => {
			const mockRepos = [createMockRepo({ id: 1 }), createMockRepo({ id: 2 })];
			vi.mocked(invoke).mockResolvedValueOnce(mockRepos);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			expect(repoLibrary.repos).toHaveLength(2);
			expect(repoLibrary.repos[0].id).toBe(1);
		});

		it('should set isLoading during load', async () => {
			let resolveInvoke: (value: unknown) => void;
			const invokePromise = new Promise((resolve) => {
				resolveInvoke = resolve;
			});
			vi.mocked(invoke).mockReturnValueOnce(invokePromise as Promise<unknown>);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const loadPromise = repoLibrary.loadRepos();

			expect(repoLibrary.isLoading).toBe(true);

			resolveInvoke!([]);
			await loadPromise;

			expect(repoLibrary.isLoading).toBe(false);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Network error'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			expect(repoLibrary.error).toContain('Network error');
			expect(repoLibrary.isLoading).toBe(false);
		});
	});

	describe('loadItems', () => {
		it('should load all items when no repoId provided', async () => {
			const mockItems = [
				createMockRepoItem({ id: 1, itemType: 'mcp' }),
				createMockRepoItem({ id: 2, itemType: 'skill' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();

			expect(repoLibrary.items).toHaveLength(2);
			expect(invoke).toHaveBeenCalledWith('get_all_repo_items', { itemType: null });
		});

		it('should load items for specific repo', async () => {
			const mockItems = [createMockRepoItem({ id: 1, repoId: 5 })];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems(5);

			expect(invoke).toHaveBeenCalledWith('get_repo_items', { repoId: 5 });
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();

			expect(repoLibrary.error).toContain('Load failed');
			expect(repoLibrary.isLoading).toBe(false);
		});
	});

	describe('loadItemsByType', () => {
		it('should load items filtered by type', async () => {
			const mockItems = [createMockRepoItem({ id: 1, itemType: 'skill' })];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItemsByType('skill');

			expect(invoke).toHaveBeenCalledWith('get_all_repo_items', { itemType: 'skill' });
			expect(repoLibrary.items).toHaveLength(1);
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Type load failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItemsByType('mcp');

			expect(repoLibrary.error).toContain('Type load failed');
		});
	});

	describe('filteredItems', () => {
		it('should filter by search query on name', async () => {
			const mockItems = [
				createMockRepoItem({ id: 1, name: 'filesystem-mcp' }),
				createMockRepoItem({ id: 2, name: 'github-skill' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			repoLibrary.setSearch('github');

			expect(repoLibrary.filteredItems).toHaveLength(1);
			expect(repoLibrary.filteredItems[0].name).toBe('github-skill');
		});

		it('should filter by search query on description', async () => {
			const mockItems = [
				createMockRepoItem({ id: 1, name: 'item-a', description: 'File operations' }),
				createMockRepoItem({ id: 2, name: 'item-b', description: 'Git integration' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			repoLibrary.setSearch('git');

			expect(repoLibrary.filteredItems).toHaveLength(1);
			expect(repoLibrary.filteredItems[0].name).toBe('item-b');
		});

		it('should filter by selected type', async () => {
			const mockItems = [
				createMockRepoItem({ id: 1, itemType: 'mcp' }),
				createMockRepoItem({ id: 2, itemType: 'skill' }),
				createMockRepoItem({ id: 3, itemType: 'mcp' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			repoLibrary.setTypeFilter('mcp');

			expect(repoLibrary.filteredItems).toHaveLength(2);
		});

		it('should apply combined search and type filters', async () => {
			const mockItems = [
				createMockRepoItem({ id: 1, itemType: 'mcp', name: 'fs-mcp' }),
				createMockRepoItem({ id: 2, itemType: 'skill', name: 'fs-skill' }),
				createMockRepoItem({ id: 3, itemType: 'mcp', name: 'git-mcp' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			repoLibrary.setSearch('fs');
			repoLibrary.setTypeFilter('mcp');

			expect(repoLibrary.filteredItems).toHaveLength(1);
			expect(repoLibrary.filteredItems[0].name).toBe('fs-mcp');
		});

		it('should return all items when no filters', async () => {
			const mockItems = [
				createMockRepoItem({ id: 1 }),
				createMockRepoItem({ id: 2 }),
				createMockRepoItem({ id: 3 })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			repoLibrary.setSearch('');
			repoLibrary.setTypeFilter('all');

			expect(repoLibrary.filteredItems).toHaveLength(3);
		});
	});

	describe('addRepo', () => {
		it('should add repo and update local state', async () => {
			const newRepo = createMockRepo({ id: 10, name: 'new-repo' });
			vi.mocked(invoke).mockResolvedValueOnce(newRepo);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const result = await repoLibrary.addRepo({
				githubUrl: 'https://github.com/test/repo',
				repoType: 'file_based',
				contentType: 'mcp'
			});

			expect(result.id).toBe(10);
			expect(repoLibrary.repos).toHaveLength(1);
		});
	});

	describe('removeRepo', () => {
		it('should remove repo and associated items', async () => {
			const repos = [createMockRepo({ id: 1 }), createMockRepo({ id: 2 })];
			const items = [
				createMockRepoItem({ id: 10, repoId: 1 }),
				createMockRepoItem({ id: 11, repoId: 2 })
			];

			vi.mocked(invoke)
				.mockResolvedValueOnce(repos) // loadRepos
				.mockResolvedValueOnce(items) // loadItems
				.mockResolvedValueOnce(undefined); // removeRepo

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();
			await repoLibrary.loadItems();
			await repoLibrary.removeRepo(1);

			expect(repoLibrary.repos).toHaveLength(1);
			expect(repoLibrary.repos[0].id).toBe(2);
			expect(repoLibrary.items).toHaveLength(1);
			expect(repoLibrary.items[0].repoId).toBe(2);
		});
	});

	describe('toggleRepo', () => {
		it('should toggle repo enabled state', async () => {
			const repos = [createMockRepo({ id: 1, isEnabled: true })];
			vi.mocked(invoke)
				.mockResolvedValueOnce(repos)
				.mockResolvedValueOnce(undefined);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();
			await repoLibrary.toggleRepo(1, false);

			expect(repoLibrary.repos[0].isEnabled).toBe(false);
		});
	});

	describe('syncRepo', () => {
		it('should set isSyncing during sync', async () => {
			const repos = [createMockRepo({ id: 1 })];
			let resolveSyncInvoke: (value: unknown) => void;
			const syncPromise = new Promise((resolve) => {
				resolveSyncInvoke = resolve;
			});

			vi.mocked(invoke)
				.mockResolvedValueOnce(repos) // loadRepos
				.mockReturnValueOnce(syncPromise as Promise<unknown>); // syncRepo

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			const promise = repoLibrary.syncRepo(1);
			expect(repoLibrary.isSyncing).toBe(true);

			// Resolve sync, then mock loadItems call
			vi.mocked(invoke).mockResolvedValueOnce([]); // loadItems after sync
			resolveSyncInvoke!(createMockSyncResult({ added: 2 }));
			const result = await promise;

			expect(repoLibrary.isSyncing).toBe(false);
			expect(result.added).toBe(2);
		});
	});

	describe('importItem', () => {
		it('should import item and reload MCP library', async () => {
			const items = [createMockRepoItem({ id: 1, itemType: 'mcp' })];
			const importResult = createMockImportResult({ itemType: 'mcp', itemId: 99 });

			vi.mocked(invoke)
				.mockResolvedValueOnce(items) // loadItems
				.mockResolvedValueOnce(importResult) // importItem
				.mockResolvedValueOnce([]); // mcpLibrary.load()

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			const result = await repoLibrary.importItem(1);

			expect(result.success).toBe(true);
			expect(repoLibrary.items[0].isImported).toBe(true);
			expect(repoLibrary.items[0].importedItemId).toBe(99);
		});

		it('should reload skill library for skill imports', async () => {
			const items = [createMockRepoItem({ id: 1, itemType: 'skill' })];
			const importResult = createMockImportResult({ itemType: 'skill', itemId: 50 });

			vi.mocked(invoke)
				.mockResolvedValueOnce(items) // loadItems
				.mockResolvedValueOnce(importResult) // importItem
				.mockResolvedValueOnce([]); // skillLibrary.load()

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			const result = await repoLibrary.importItem(1);

			expect(result.success).toBe(true);
		});

		it('should reload subagent library for subagent imports', async () => {
			const items = [createMockRepoItem({ id: 1, itemType: 'subagent' })];
			const importResult = createMockImportResult({ itemType: 'subagent', itemId: 30 });

			vi.mocked(invoke)
				.mockResolvedValueOnce(items) // loadItems
				.mockResolvedValueOnce(importResult) // importItem
				.mockResolvedValueOnce([]); // subagentLibrary.load()

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();
			const result = await repoLibrary.importItem(1);

			expect(result.success).toBe(true);
		});
	});

	describe('searchRegistry', () => {
		it('should populate registryMcps with deduplication', async () => {
			const registryMcps = [
				createMockRegistryMcp({ registryId: 'a', name: 'mcp-a' }),
				createMockRegistryMcp({ registryId: 'b', name: 'mcp-b' }),
				createMockRegistryMcp({ registryId: 'a', name: 'mcp-a-dup' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(registryMcps);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.searchRegistry('test');

			expect(repoLibrary.registryMcps).toHaveLength(2);
			expect(repoLibrary.registrySearchQuery).toBe('test');
		});

		it('should handle search errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Search failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.searchRegistry('test');

			expect(repoLibrary.registryError).toContain('Search failed');
			expect(repoLibrary.isSearchingRegistry).toBe(false);
		});
	});

	describe('loadRegistryMcps', () => {
		it('should load initial registry MCPs', async () => {
			const result = {
				entries: [createMockRegistryMcp({ registryId: 'a' })],
				nextCursor: 'cursor-1'
			};
			vi.mocked(invoke).mockResolvedValueOnce(result);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();

			expect(repoLibrary.registryMcps).toHaveLength(1);
			expect(repoLibrary.registryNextCursor).toBe('cursor-1');
		});

		it('should append and deduplicate on loadMore', async () => {
			const result1 = {
				entries: [createMockRegistryMcp({ registryId: 'a', name: 'mcp-a' })],
				nextCursor: 'cursor-1'
			};
			const result2 = {
				entries: [
					createMockRegistryMcp({ registryId: 'a', name: 'mcp-a' }),
					createMockRegistryMcp({ registryId: 'b', name: 'mcp-b' })
				],
				nextCursor: 'cursor-2'
			};

			vi.mocked(invoke)
				.mockResolvedValueOnce(result1)
				.mockResolvedValueOnce(result2);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();
			await repoLibrary.loadRegistryMcps(true);

			expect(repoLibrary.registryMcps).toHaveLength(2);
		});

		it('should clear cursor when loadMore returns only duplicates', async () => {
			const result1 = {
				entries: [createMockRegistryMcp({ registryId: 'a' })],
				nextCursor: 'cursor-1'
			};
			const result2 = {
				entries: [createMockRegistryMcp({ registryId: 'a' })],
				nextCursor: 'cursor-2'
			};

			vi.mocked(invoke)
				.mockResolvedValueOnce(result1)
				.mockResolvedValueOnce(result2);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();
			await repoLibrary.loadRegistryMcps(true);

			expect(repoLibrary.registryNextCursor).toBeNull();
		});

		it('should handle errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Registry error'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();

			expect(repoLibrary.registryError).toContain('Registry error');
			expect(repoLibrary.isSearchingRegistry).toBe(false);
		});
	});

	describe('getRepoById / getItemById', () => {
		it('should return repo by id', async () => {
			const repos = [createMockRepo({ id: 1, name: 'test-repo' })];
			vi.mocked(invoke).mockResolvedValueOnce(repos);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			expect(repoLibrary.getRepoById(1)?.name).toBe('test-repo');
			expect(repoLibrary.getRepoById(999)).toBeUndefined();
		});

		it('should return item by id', async () => {
			const items = [createMockRepoItem({ id: 5, name: 'test-item' })];
			vi.mocked(invoke).mockResolvedValueOnce(items);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();

			expect(repoLibrary.getItemById(5)?.name).toBe('test-item');
			expect(repoLibrary.getItemById(999)).toBeUndefined();
		});
	});
});
