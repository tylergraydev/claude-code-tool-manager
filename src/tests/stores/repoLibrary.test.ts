import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import type { Repo, RepoItem, RegistryMcpEntry, RegistrySearchResult } from '$lib/types';

// Mock the other libraries
vi.mock('$lib/stores/mcpLibrary.svelte', () => ({
	mcpLibrary: {
		load: vi.fn().mockResolvedValue(undefined)
	}
}));

vi.mock('$lib/stores/skillLibrary.svelte', () => ({
	skillLibrary: {
		load: vi.fn().mockResolvedValue(undefined)
	}
}));

vi.mock('$lib/stores/subagentLibrary.svelte', () => ({
	subagentLibrary: {
		load: vi.fn().mockResolvedValue(undefined)
	}
}));

describe('RepoLibrary Store', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		vi.resetModules();
	});

	const createMockRepo = (overrides: Partial<Repo> = {}): Repo => ({
		id: 1,
		owner: 'test-owner',
		name: 'test-repo',
		isEnabled: true,
		lastFetchedAt: null,
		createdAt: '2024-01-01T00:00:00Z',
		...overrides
	});

	const createMockItem = (overrides: Partial<RepoItem> = {}): RepoItem => ({
		id: 1,
		repoId: 1,
		name: 'test-item',
		description: 'Test description',
		itemType: 'mcp',
		filePath: '/test/path.json',
		sha: 'abc123',
		isImported: false,
		importedItemId: null,
		createdAt: '2024-01-01T00:00:00Z',
		updatedAt: '2024-01-01T00:00:00Z',
		...overrides
	});

	const createMockRegistryEntry = (overrides: Partial<RegistryMcpEntry> = {}): RegistryMcpEntry => ({
		registryId: 'test-mcp',
		name: 'Test MCP',
		description: 'A test MCP',
		vendor: 'test-vendor',
		sourceUrl: 'https://github.com/test/mcp',
		homepage: 'https://test-mcp.com',
		license: 'MIT',
		...overrides
	});

	describe('initial state', () => {
		it('should have correct initial values', async () => {
			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');

			expect(repoLibrary.repos).toEqual([]);
			expect(repoLibrary.items).toEqual([]);
			expect(repoLibrary.isLoading).toBe(false);
			expect(repoLibrary.isSyncing).toBe(false);
			expect(repoLibrary.error).toBeNull();
			expect(repoLibrary.searchQuery).toBe('');
			expect(repoLibrary.selectedType).toBe('all');
			expect(repoLibrary.rateLimitInfo).toBeNull();
			expect(repoLibrary.registryMcps).toEqual([]);
			expect(repoLibrary.registrySearchQuery).toBe('');
			expect(repoLibrary.registryNextCursor).toBeNull();
			expect(repoLibrary.isSearchingRegistry).toBe(false);
			expect(repoLibrary.registryError).toBeNull();
		});
	});

	describe('loadRepos', () => {
		it('should load repos successfully', async () => {
			const mockRepos = [
				createMockRepo({ id: 1, name: 'repo1' }),
				createMockRepo({ id: 2, name: 'repo2' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockRepos);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			expect(invoke).toHaveBeenCalledWith('get_all_repos');
			expect(repoLibrary.repos).toEqual(mockRepos);
			expect(repoLibrary.isLoading).toBe(false);
		});

		it('should handle load errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			expect(repoLibrary.error).toBe('Error: Load failed');
			expect(repoLibrary.isLoading).toBe(false);
		});
	});

	describe('loadItems', () => {
		it('should load all items when no repoId provided', async () => {
			const mockItems = [
				createMockItem({ id: 1, name: 'item1' }),
				createMockItem({ id: 2, name: 'item2' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();

			expect(invoke).toHaveBeenCalledWith('get_all_repo_items', { itemType: null });
			expect(repoLibrary.items).toEqual(mockItems);
		});

		it('should load items for specific repo when repoId provided', async () => {
			const mockItems = [createMockItem({ id: 1, repoId: 5 })];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems(5);

			expect(invoke).toHaveBeenCalledWith('get_repo_items', { repoId: 5 });
			expect(repoLibrary.items).toEqual(mockItems);
		});

		it('should handle load errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load items failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();

			expect(repoLibrary.error).toBe('Error: Load items failed');
		});
	});

	describe('loadItemsByType', () => {
		it('should load items by type', async () => {
			const mockItems = [createMockItem({ id: 1, itemType: 'skill' })];
			vi.mocked(invoke).mockResolvedValueOnce(mockItems);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItemsByType('skill');

			expect(invoke).toHaveBeenCalledWith('get_all_repo_items', { itemType: 'skill' });
			expect(repoLibrary.items).toEqual(mockItems);
		});

		it('should handle load errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Type load failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItemsByType('mcp');

			expect(repoLibrary.error).toBe('Error: Type load failed');
		});
	});

	describe('addRepo', () => {
		it('should add repo and update state', async () => {
			const newRepo = createMockRepo({ id: 3, name: 'new-repo' });
			vi.mocked(invoke).mockResolvedValueOnce(newRepo);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const result = await repoLibrary.addRepo({ owner: 'test', name: 'new-repo' });

			expect(invoke).toHaveBeenCalledWith('add_repo', { request: { owner: 'test', name: 'new-repo' } });
			expect(result).toEqual(newRepo);
			expect(repoLibrary.repos).toContainEqual(newRepo);
		});
	});

	describe('removeRepo', () => {
		it('should remove repo and its items from state', async () => {
			const repo1 = createMockRepo({ id: 1, name: 'repo1' });
			const repo2 = createMockRepo({ id: 2, name: 'repo2' });
			const item1 = createMockItem({ id: 1, repoId: 1 });
			const item2 = createMockItem({ id: 2, repoId: 2 });

			vi.mocked(invoke)
				.mockResolvedValueOnce([repo1, repo2])
				.mockResolvedValueOnce([item1, item2])
				.mockResolvedValueOnce(undefined);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();
			await repoLibrary.loadItems();

			await repoLibrary.removeRepo(1);

			expect(invoke).toHaveBeenCalledWith('remove_repo', { id: 1 });
			expect(repoLibrary.repos).not.toContainEqual(repo1);
			expect(repoLibrary.repos).toContainEqual(repo2);
			expect(repoLibrary.items).not.toContainEqual(item1);
			expect(repoLibrary.items).toContainEqual(item2);
		});
	});

	describe('toggleRepo', () => {
		it('should toggle repo enabled state', async () => {
			const repo = createMockRepo({ id: 1, isEnabled: true });
			vi.mocked(invoke)
				.mockResolvedValueOnce([repo])
				.mockResolvedValueOnce(undefined);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();
			await repoLibrary.toggleRepo(1, false);

			expect(invoke).toHaveBeenCalledWith('toggle_repo', { id: 1, enabled: false });
			expect(repoLibrary.repos[0].isEnabled).toBe(false);
		});
	});

	describe('syncRepo', () => {
		it('should sync repo and reload items', async () => {
			const syncResult = { itemsAdded: 5, itemsUpdated: 2, itemsRemoved: 1 };
			vi.mocked(invoke)
				.mockResolvedValueOnce(syncResult)
				.mockResolvedValueOnce([]);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');

			// Set up initial repos
			repoLibrary.repos = [createMockRepo({ id: 1 })];

			const result = await repoLibrary.syncRepo(1);

			expect(invoke).toHaveBeenCalledWith('sync_repo', { id: 1 });
			expect(result).toEqual(syncResult);
			expect(repoLibrary.isSyncing).toBe(false);
			expect(repoLibrary.repos[0].lastFetchedAt).not.toBeNull();
		});

		it('should handle sync errors and reset isSyncing', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Sync failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.repos = [createMockRepo({ id: 1 })];

			await expect(repoLibrary.syncRepo(1)).rejects.toThrow('Sync failed');
			expect(repoLibrary.isSyncing).toBe(false);
		});
	});

	describe('syncAllRepos', () => {
		it('should sync all repos and reload data', async () => {
			const syncResult = { itemsAdded: 10, itemsUpdated: 5, itemsRemoved: 2 };
			vi.mocked(invoke)
				.mockResolvedValueOnce(syncResult)
				.mockResolvedValueOnce([])
				.mockResolvedValueOnce([]);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const result = await repoLibrary.syncAllRepos();

			expect(invoke).toHaveBeenCalledWith('sync_all_repos');
			expect(result).toEqual(syncResult);
			expect(repoLibrary.isSyncing).toBe(false);
		});
	});

	describe('importItem', () => {
		it('should import MCP item and reload MCP library', async () => {
			const importResult = { success: true, itemId: 10, itemType: 'mcp' as const };
			vi.mocked(invoke).mockResolvedValueOnce(importResult);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');

			repoLibrary.items = [createMockItem({ id: 1, itemType: 'mcp', isImported: false })];

			const result = await repoLibrary.importItem(1);

			expect(invoke).toHaveBeenCalledWith('import_repo_item', { itemId: 1 });
			expect(result).toEqual(importResult);
			expect(repoLibrary.items[0].isImported).toBe(true);
			expect(repoLibrary.items[0].importedItemId).toBe(10);
			expect(mcpLibrary.load).toHaveBeenCalled();
		});

		it('should import skill item and reload skill library', async () => {
			const importResult = { success: true, itemId: 20, itemType: 'skill' as const };
			vi.mocked(invoke).mockResolvedValueOnce(importResult);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const { skillLibrary } = await import('$lib/stores/skillLibrary.svelte');

			repoLibrary.items = [createMockItem({ id: 2, itemType: 'skill', isImported: false })];

			await repoLibrary.importItem(2);

			expect(skillLibrary.load).toHaveBeenCalled();
		});

		it('should import subagent item and reload subagent library', async () => {
			const importResult = { success: true, itemId: 30, itemType: 'subagent' as const };
			vi.mocked(invoke).mockResolvedValueOnce(importResult);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const { subagentLibrary } = await import('$lib/stores/subagentLibrary.svelte');

			repoLibrary.items = [createMockItem({ id: 3, itemType: 'subagent', isImported: false })];

			await repoLibrary.importItem(3);

			expect(subagentLibrary.load).toHaveBeenCalled();
		});

		it('should not update state on failed import', async () => {
			const importResult = { success: false, itemId: null, itemType: 'mcp' as const };
			vi.mocked(invoke).mockResolvedValueOnce(importResult);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [createMockItem({ id: 1, isImported: false })];

			await repoLibrary.importItem(1);

			expect(repoLibrary.items[0].isImported).toBe(false);
		});
	});

	describe('checkRateLimit', () => {
		it('should fetch rate limit info', async () => {
			const rateLimitInfo = { limit: 60, remaining: 45, resetAt: '2024-01-01T01:00:00Z' };
			vi.mocked(invoke).mockResolvedValueOnce(rateLimitInfo);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.checkRateLimit();

			expect(invoke).toHaveBeenCalledWith('get_github_rate_limit');
			expect(repoLibrary.rateLimitInfo).toEqual(rateLimitInfo);
		});

		it('should handle rate limit check errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Rate limit failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.checkRateLimit();

			// Should not throw, just log
			expect(repoLibrary.rateLimitInfo).toBeNull();
		});
	});

	describe('seedDefaultRepos', () => {
		it('should seed default repos and reload', async () => {
			vi.mocked(invoke)
				.mockResolvedValueOnce(undefined)
				.mockResolvedValueOnce([createMockRepo()]);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.seedDefaultRepos();

			expect(invoke).toHaveBeenCalledWith('seed_default_repos');
			expect(invoke).toHaveBeenCalledWith('get_all_repos');
		});

		it('should handle seed errors silently', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Seed failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.seedDefaultRepos();

			// Should not throw
		});
	});

	describe('getRepoById', () => {
		it('should return repo by id', async () => {
			const repo = createMockRepo({ id: 5 });
			vi.mocked(invoke).mockResolvedValueOnce([repo]);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRepos();

			expect(repoLibrary.getRepoById(5)).toEqual(repo);
		});

		it('should return undefined for non-existent id', async () => {
			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			expect(repoLibrary.getRepoById(999)).toBeUndefined();
		});
	});

	describe('getItemById', () => {
		it('should return item by id', async () => {
			const item = createMockItem({ id: 10 });
			vi.mocked(invoke).mockResolvedValueOnce([item]);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadItems();

			expect(repoLibrary.getItemById(10)).toEqual(item);
		});

		it('should return undefined for non-existent id', async () => {
			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			expect(repoLibrary.getItemById(999)).toBeUndefined();
		});
	});

	describe('setSearch', () => {
		it('should set search query', async () => {
			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.setSearch('test query');
			expect(repoLibrary.searchQuery).toBe('test query');
		});
	});

	describe('setTypeFilter', () => {
		it('should set type filter', async () => {
			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.setTypeFilter('mcp');
			expect(repoLibrary.selectedType).toBe('mcp');
		});
	});

	describe('filteredItems', () => {
		it('should filter items by type', async () => {
			const mcpItem = createMockItem({ id: 1, itemType: 'mcp' });
			const skillItem = createMockItem({ id: 2, itemType: 'skill' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [mcpItem, skillItem];
			repoLibrary.setTypeFilter('mcp');

			expect(repoLibrary.filteredItems).toEqual([mcpItem]);
		});

		it('should filter items by search query', async () => {
			const item1 = createMockItem({ id: 1, name: 'test-mcp', description: 'A test MCP' });
			const item2 = createMockItem({ id: 2, name: 'other-mcp', description: 'Another MCP' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [item1, item2];
			repoLibrary.setSearch('test');

			expect(repoLibrary.filteredItems).toEqual([item1]);
		});

		it('should filter by both type and search query', async () => {
			const mcpItem1 = createMockItem({ id: 1, itemType: 'mcp', name: 'test-mcp', description: 'A test item' });
			const mcpItem2 = createMockItem({ id: 2, itemType: 'mcp', name: 'other-mcp', description: 'Something else' });
			const skillItem = createMockItem({ id: 3, itemType: 'skill', name: 'test-skill', description: 'Another test' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [mcpItem1, mcpItem2, skillItem];
			repoLibrary.setTypeFilter('mcp');
			repoLibrary.setSearch('test');

			expect(repoLibrary.filteredItems).toEqual([mcpItem1]);
		});

		it('should return all items when no filters applied', async () => {
			const items = [
				createMockItem({ id: 1 }),
				createMockItem({ id: 2 })
			];

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = items;
			repoLibrary.setTypeFilter('all');
			repoLibrary.setSearch('');

			expect(repoLibrary.filteredItems).toEqual(items);
		});
	});

	describe('derived item type arrays', () => {
		it('should return mcpItems correctly', async () => {
			const mcpItem = createMockItem({ id: 1, itemType: 'mcp' });
			const skillItem = createMockItem({ id: 2, itemType: 'skill' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [mcpItem, skillItem];

			expect(repoLibrary.mcpItems).toEqual([mcpItem]);
		});

		it('should return skillItems correctly', async () => {
			const mcpItem = createMockItem({ id: 1, itemType: 'mcp' });
			const skillItem = createMockItem({ id: 2, itemType: 'skill' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [mcpItem, skillItem];

			expect(repoLibrary.skillItems).toEqual([skillItem]);
		});

		it('should return subagentItems correctly', async () => {
			const mcpItem = createMockItem({ id: 1, itemType: 'mcp' });
			const subagentItem = createMockItem({ id: 2, itemType: 'subagent' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.items = [mcpItem, subagentItem];

			expect(repoLibrary.subagentItems).toEqual([subagentItem]);
		});
	});

	describe('searchRegistry', () => {
		it('should search registry and store results', async () => {
			const entries = [
				createMockRegistryEntry({ registryId: 'mcp1' }),
				createMockRegistryEntry({ registryId: 'mcp2' })
			];
			vi.mocked(invoke).mockResolvedValueOnce(entries);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.searchRegistry('test');

			expect(invoke).toHaveBeenCalledWith('search_mcp_registry', { query: 'test', limit: 50 });
			expect(repoLibrary.registryMcps).toEqual(entries);
			expect(repoLibrary.registrySearchQuery).toBe('test');
			expect(repoLibrary.isSearchingRegistry).toBe(false);
		});

		it('should deduplicate results', async () => {
			const entry = createMockRegistryEntry({ registryId: 'mcp1' });
			const entries = [entry, entry];
			vi.mocked(invoke).mockResolvedValueOnce(entries);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.searchRegistry('test');

			expect(repoLibrary.registryMcps).toHaveLength(1);
		});

		it('should handle search errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Search failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.searchRegistry('test');

			expect(repoLibrary.registryError).toBe('Error: Search failed');
			expect(repoLibrary.isSearchingRegistry).toBe(false);
		});
	});

	describe('loadRegistryMcps', () => {
		it('should load registry MCPs', async () => {
			const result: RegistrySearchResult = {
				entries: [createMockRegistryEntry({ registryId: 'mcp1' })],
				nextCursor: 'cursor123'
			};
			vi.mocked(invoke).mockResolvedValueOnce(result);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();

			expect(invoke).toHaveBeenCalledWith('list_mcp_registry', { cursor: null });
			expect(repoLibrary.registryMcps).toEqual(result.entries);
			expect(repoLibrary.registryNextCursor).toBe('cursor123');
		});

		it('should load more results when loadMore is true', async () => {
			const firstResult: RegistrySearchResult = {
				entries: [createMockRegistryEntry({ registryId: 'mcp1' })],
				nextCursor: 'cursor1'
			};
			const secondResult: RegistrySearchResult = {
				entries: [createMockRegistryEntry({ registryId: 'mcp2' })],
				nextCursor: 'cursor2'
			};
			vi.mocked(invoke)
				.mockResolvedValueOnce(firstResult)
				.mockResolvedValueOnce(secondResult);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();
			await repoLibrary.loadRegistryMcps(true);

			expect(invoke).toHaveBeenLastCalledWith('list_mcp_registry', { cursor: 'cursor1' });
			expect(repoLibrary.registryMcps).toHaveLength(2);
		});

		it('should clear cursor if no new unique items added', async () => {
			const entry = createMockRegistryEntry({ registryId: 'mcp1' });
			const firstResult: RegistrySearchResult = {
				entries: [entry],
				nextCursor: 'cursor1'
			};
			const secondResult: RegistrySearchResult = {
				entries: [entry], // same entry, will be deduplicated
				nextCursor: 'cursor2'
			};
			vi.mocked(invoke)
				.mockResolvedValueOnce(firstResult)
				.mockResolvedValueOnce(secondResult);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();
			await repoLibrary.loadRegistryMcps(true);

			// Cursor should be cleared because no new items were added
			expect(repoLibrary.registryNextCursor).toBeNull();
		});

		it('should handle load errors', async () => {
			vi.mocked(invoke).mockRejectedValueOnce(new Error('Load failed'));

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			await repoLibrary.loadRegistryMcps();

			expect(repoLibrary.registryError).toBe('Error: Load failed');
		});
	});

	describe('importFromRegistry', () => {
		it('should import from registry and reload MCP library', async () => {
			vi.mocked(invoke).mockResolvedValueOnce(42);

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			const { mcpLibrary } = await import('$lib/stores/mcpLibrary.svelte');

			const entry = createMockRegistryEntry();
			const result = await repoLibrary.importFromRegistry(entry);

			expect(invoke).toHaveBeenCalledWith('import_mcp_from_registry', { entry });
			expect(result).toBe(42);
			expect(mcpLibrary.load).toHaveBeenCalled();
		});
	});

	describe('clearRegistrySearch', () => {
		it('should clear all registry state', async () => {
			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');

			repoLibrary.registrySearchQuery = 'test';
			repoLibrary.registryMcps = [createMockRegistryEntry()];
			repoLibrary.registryNextCursor = 'cursor123';
			repoLibrary.registryError = 'Some error';

			repoLibrary.clearRegistrySearch();

			expect(repoLibrary.registrySearchQuery).toBe('');
			expect(repoLibrary.registryMcps).toEqual([]);
			expect(repoLibrary.registryNextCursor).toBeNull();
			expect(repoLibrary.registryError).toBeNull();
		});
	});

	describe('filteredRegistryMcps', () => {
		it('should return all MCPs when no search query', async () => {
			const entries = [
				createMockRegistryEntry({ registryId: 'mcp1' }),
				createMockRegistryEntry({ registryId: 'mcp2' })
			];

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.registryMcps = entries;
			repoLibrary.registrySearchQuery = '';

			expect(repoLibrary.filteredRegistryMcps).toEqual(entries);
		});

		it('should filter by name', async () => {
			const entry1 = createMockRegistryEntry({ registryId: 'mcp1', name: 'Search Match MCP', description: 'Some MCP' });
			const entry2 = createMockRegistryEntry({ registryId: 'mcp2', name: 'Other MCP', description: 'Another MCP' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.registryMcps = [entry1, entry2];
			repoLibrary.registrySearchQuery = 'search';

			expect(repoLibrary.filteredRegistryMcps).toEqual([entry1]);
		});

		it('should filter by description', async () => {
			const entry1 = createMockRegistryEntry({ registryId: 'mcp1', name: 'First MCP', description: 'Contains unique keyword' });
			const entry2 = createMockRegistryEntry({ registryId: 'mcp2', name: 'Second MCP', description: 'Different text here' });

			const { repoLibrary } = await import('$lib/stores/repoLibrary.svelte');
			repoLibrary.registryMcps = [entry1, entry2];
			repoLibrary.registrySearchQuery = 'unique';

			expect(repoLibrary.filteredRegistryMcps).toEqual([entry1]);
		});
	});
});
