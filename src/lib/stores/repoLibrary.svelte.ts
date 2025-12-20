import { invoke } from '@tauri-apps/api/core';
import type {
	Repo,
	RepoItem,
	CreateRepoRequest,
	SyncResult,
	RateLimitInfo,
	ImportResult,
	ItemType,
	RegistryMcpEntry,
	RegistrySearchResult
} from '$lib/types';
import { mcpLibrary } from './mcpLibrary.svelte';
import { skillLibrary } from './skillLibrary.svelte';
import { subagentLibrary } from './subagentLibrary.svelte';

class RepoLibraryState {
	repos = $state<Repo[]>([]);
	items = $state<RepoItem[]>([]);
	isLoading = $state(false);
	isSyncing = $state(false);
	error = $state<string | null>(null);
	searchQuery = $state('');
	selectedType = $state<'all' | ItemType>('all');
	rateLimitInfo = $state<RateLimitInfo | null>(null);

	// MCP Registry state
	registryMcps = $state<RegistryMcpEntry[]>([]);
	registrySearchQuery = $state('');
	registryNextCursor = $state<string | null>(null);
	isSearchingRegistry = $state(false);
	registryError = $state<string | null>(null);

	filteredItems = $derived.by(() => {
		let result = this.items;

		// Filter by type
		if (this.selectedType !== 'all') {
			result = result.filter((i) => i.itemType === this.selectedType);
		}

		// Filter by search query
		if (this.searchQuery) {
			const query = this.searchQuery.toLowerCase();
			result = result.filter(
				(i) =>
					i.name.toLowerCase().includes(query) ||
					i.description?.toLowerCase().includes(query)
			);
		}

		return result;
	});

	mcpItems = $derived(this.items.filter((i) => i.itemType === 'mcp'));
	skillItems = $derived(this.items.filter((i) => i.itemType === 'skill'));
	subagentItems = $derived(this.items.filter((i) => i.itemType === 'subagent'));

	async loadRepos() {
		this.isLoading = true;
		this.error = null;
		try {
			this.repos = await invoke<Repo[]>('get_all_repos');
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load repos:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadItems(repoId?: number) {
		this.isLoading = true;
		this.error = null;
		try {
			if (repoId) {
				this.items = await invoke<RepoItem[]>('get_repo_items', { repoId });
			} else {
				this.items = await invoke<RepoItem[]>('get_all_repo_items', { itemType: null });
			}
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load repo items:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async loadItemsByType(itemType: ItemType) {
		this.isLoading = true;
		this.error = null;
		try {
			this.items = await invoke<RepoItem[]>('get_all_repo_items', { itemType });
		} catch (e) {
			this.error = String(e);
			console.error('Failed to load repo items:', e);
		} finally {
			this.isLoading = false;
		}
	}

	async addRepo(request: CreateRepoRequest): Promise<Repo> {
		const repo = await invoke<Repo>('add_repo', { request });
		this.repos = [...this.repos, repo];
		return repo;
	}

	async removeRepo(id: number): Promise<void> {
		await invoke('remove_repo', { id });
		this.repos = this.repos.filter((r) => r.id !== id);
		// Also remove items from this repo
		this.items = this.items.filter((i) => i.repoId !== id);
	}

	async toggleRepo(id: number, enabled: boolean): Promise<void> {
		await invoke('toggle_repo', { id, enabled });
		this.repos = this.repos.map((r) => (r.id === id ? { ...r, isEnabled: enabled } : r));
	}

	async syncRepo(id: number): Promise<SyncResult> {
		this.isSyncing = true;
		try {
			const result = await invoke<SyncResult>('sync_repo', { id });
			// Reload items after sync
			await this.loadItems();
			// Update repo's lastFetchedAt
			this.repos = this.repos.map((r) =>
				r.id === id ? { ...r, lastFetchedAt: new Date().toISOString() } : r
			);
			return result;
		} finally {
			this.isSyncing = false;
		}
	}

	async syncAllRepos(): Promise<SyncResult> {
		this.isSyncing = true;
		try {
			const result = await invoke<SyncResult>('sync_all_repos');
			// Reload everything after sync
			await this.loadRepos();
			await this.loadItems();
			return result;
		} finally {
			this.isSyncing = false;
		}
	}

	async importItem(itemId: number): Promise<ImportResult> {
		const result = await invoke<ImportResult>('import_repo_item', { itemId });
		if (result.success) {
			// Update item's imported status
			this.items = this.items.map((i) =>
				i.id === itemId ? { ...i, isImported: true, importedItemId: result.itemId } : i
			);

			// Reload the appropriate library so the imported item shows up
			if (result.itemType === 'mcp') {
				await mcpLibrary.load();
			} else if (result.itemType === 'skill') {
				await skillLibrary.load();
			} else if (result.itemType === 'subagent') {
				await subagentLibrary.load();
			}
		}
		return result;
	}

	async checkRateLimit(): Promise<void> {
		try {
			this.rateLimitInfo = await invoke<RateLimitInfo>('get_github_rate_limit');
		} catch (e) {
			console.error('Failed to check rate limit:', e);
		}
	}

	async seedDefaultRepos(): Promise<void> {
		try {
			await invoke('seed_default_repos');
			await this.loadRepos();
		} catch (e) {
			console.error('Failed to seed default repos:', e);
		}
	}

	getRepoById(id: number): Repo | undefined {
		return this.repos.find((r) => r.id === id);
	}

	getItemById(id: number): RepoItem | undefined {
		return this.items.find((i) => i.id === id);
	}

	setSearch(query: string) {
		this.searchQuery = query;
	}

	setTypeFilter(type: 'all' | ItemType) {
		this.selectedType = type;
	}

	// ===== MCP Registry Methods =====

	async searchRegistry(query: string): Promise<void> {
		this.isSearchingRegistry = true;
		this.registryError = null;
		this.registrySearchQuery = query;
		try {
			this.registryMcps = await invoke<RegistryMcpEntry[]>('search_mcp_registry', {
				query,
				limit: 50
			});
			this.registryNextCursor = null;
		} catch (e) {
			this.registryError = String(e);
			console.error('Failed to search registry:', e);
		} finally {
			this.isSearchingRegistry = false;
		}
	}

	async loadRegistryMcps(loadMore = false): Promise<void> {
		this.isSearchingRegistry = true;
		this.registryError = null;
		try {
			const result = await invoke<RegistrySearchResult>('list_mcp_registry', {
				limit: 50,
				cursor: loadMore ? this.registryNextCursor : null
			});
			if (loadMore) {
				this.registryMcps = [...this.registryMcps, ...result.entries];
			} else {
				this.registryMcps = result.entries;
			}
			this.registryNextCursor = result.nextCursor ?? null;
		} catch (e) {
			this.registryError = String(e);
			console.error('Failed to load registry MCPs:', e);
		} finally {
			this.isSearchingRegistry = false;
		}
	}

	async importFromRegistry(entry: RegistryMcpEntry): Promise<number> {
		const id = await invoke<number>('import_mcp_from_registry', { entry });
		// Reload MCP library so the imported MCP shows up
		await mcpLibrary.load();
		return id;
	}

	clearRegistrySearch(): void {
		this.registrySearchQuery = '';
		this.registryMcps = [];
		this.registryNextCursor = null;
		this.registryError = null;
	}

	get filteredRegistryMcps(): RegistryMcpEntry[] {
		if (!this.registrySearchQuery) {
			return this.registryMcps;
		}
		const query = this.registrySearchQuery.toLowerCase();
		return this.registryMcps.filter(
			(m) =>
				m.name.toLowerCase().includes(query) ||
				m.description?.toLowerCase().includes(query)
		);
	}
}

export const repoLibrary = new RepoLibraryState();
