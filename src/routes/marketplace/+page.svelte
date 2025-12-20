<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Header } from '$lib/components/layout';
	import { repoLibrary, notifications } from '$lib/stores';
	import {
		RefreshCw,
		Plus,
		Store,
		FileCode,
		Bot,
		ExternalLink,
		Trash2,
		Download,
		Github,
		X,
		Eye,
		RotateCcw,
		Search,
		Server,
		Package,
		Loader2
	} from 'lucide-svelte';
	import type { Repo, RepoItem, CreateRepoRequest, ItemType, RegistryMcpEntry } from '$lib/types';

	// State
	let activeTab = $state<'mcps' | 'repos' | 'skills' | 'agents'>('mcps');
	let showAddRepoModal = $state(false);
	let newRepoUrl = $state('');
	let newRepoType = $state<'file_based' | 'readme_based'>('readme_based');
	let newRepoContentType = $state<'skill' | 'subagent' | 'mixed'>('mixed');
	let selectedItem = $state<RepoItem | null>(null);

	// MCP Registry state
	let mcpSearchQuery = $state('');
	let selectedRegistryMcp = $state<RegistryMcpEntry | null>(null);
	let isImportingMcp = $state(false);

	// Load data on mount and auto-sync if no items
	$effect(() => {
		loadData();
	});

	// Load MCP Registry when switching to MCPs tab
	$effect(() => {
		if (activeTab === 'mcps' && repoLibrary.registryMcps.length === 0 && !repoLibrary.isSearchingRegistry) {
			repoLibrary.loadRegistryMcps();
		}
	});

	async function loadData() {
		await repoLibrary.loadRepos();
		await repoLibrary.loadItems();
		repoLibrary.checkRateLimit();

		// Auto-sync if no items have been fetched yet
		if (repoLibrary.repos.length > 0 && repoLibrary.items.length === 0) {
			notifications.info('Syncing repositories for the first time...');
			await handleSyncAll();
		}
	}

	async function handleSyncAll() {
		try {
			const result = await repoLibrary.syncAllRepos();
			if (result.errors.length > 0) {
				notifications.warning(`Synced with ${result.errors.length} errors`);
			} else {
				notifications.success(`Added ${result.added}, updated ${result.updated} items`);
			}
		} catch (e) {
			notifications.error('Failed to sync repositories');
		}
	}

	async function handleSyncRepo(id: number) {
		try {
			const result = await repoLibrary.syncRepo(id);
			notifications.success(`Added ${result.added}, updated ${result.updated} items`);
		} catch (e) {
			notifications.error('Failed to sync repository');
		}
	}

	async function handleAddRepo() {
		if (!newRepoUrl) return;

		try {
			const request: CreateRepoRequest = {
				githubUrl: newRepoUrl,
				repoType: newRepoType,
				contentType: newRepoContentType
			};
			await repoLibrary.addRepo(request);
			notifications.success('Repository added');
			showAddRepoModal = false;
			newRepoUrl = '';
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleRemoveRepo(repo: Repo) {
		if (repo.isDefault) {
			notifications.error('Cannot remove default repositories');
			return;
		}
		try {
			await repoLibrary.removeRepo(repo.id);
			notifications.success('Repository removed');
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleToggleRepo(repo: Repo) {
		try {
			await repoLibrary.toggleRepo(repo.id, !repo.isEnabled);
		} catch (e) {
			notifications.error('Failed to toggle repository');
		}
	}

	async function handleImport(item: RepoItem) {
		try {
			const result = await repoLibrary.importItem(item.id);
			if (result.success) {
				notifications.success(`Imported ${item.name}`);
			} else {
				notifications.warning(result.message || 'Already imported');
			}
		} catch (e) {
			notifications.error('Failed to import item');
		}
	}

	async function handleResetRepos() {
		try {
			await invoke('reset_repos_to_defaults');
			await repoLibrary.loadRepos();
			await repoLibrary.loadItems();
			notifications.success('Repos reset to defaults');
			await handleSyncAll();
		} catch (e) {
			notifications.error('Failed to reset repos');
		}
	}

	function getFilteredItems(type: ItemType): RepoItem[] {
		return repoLibrary.items.filter((i) => i.itemType === type);
	}

	// MCP Registry handlers
	async function handleMcpSearch() {
		if (!mcpSearchQuery.trim()) {
			await repoLibrary.loadRegistryMcps();
			return;
		}
		await repoLibrary.searchRegistry(mcpSearchQuery);
	}

	async function handleMcpSearchKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			await handleMcpSearch();
		}
	}

	async function handleImportRegistryMcp(entry: RegistryMcpEntry) {
		isImportingMcp = true;
		try {
			await repoLibrary.importFromRegistry(entry);
			notifications.success(`Imported ${entry.name} to your MCP Library`);
			selectedRegistryMcp = null;
		} catch (e) {
			notifications.error(`Failed to import: ${e}`);
		} finally {
			isImportingMcp = false;
		}
	}

	async function handleLoadMoreMcps() {
		await repoLibrary.loadRegistryMcps(true);
	}

	function getMcpTypeColor(type: string): string {
		switch (type) {
			case 'npm':
				return 'bg-red-100 text-red-600 dark:bg-red-900/50 dark:text-red-400';
			case 'pypi':
				return 'bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400';
			case 'docker':
			case 'oci':
				return 'bg-cyan-100 text-cyan-600 dark:bg-cyan-900/50 dark:text-cyan-400';
			default:
				return 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400';
		}
	}

	const typeIcons: Record<ItemType, typeof FileCode> = {
		mcp: FileCode, // Not used but kept for type safety
		skill: FileCode,
		subagent: Bot
	};

	const typeColors: Record<ItemType, string> = {
		mcp: 'bg-purple-100 text-purple-600 dark:bg-purple-900/50 dark:text-purple-400',
		skill: 'bg-yellow-100 text-yellow-600 dark:bg-yellow-900/50 dark:text-yellow-400',
		subagent: 'bg-cyan-100 text-cyan-600 dark:bg-cyan-900/50 dark:text-cyan-400'
	};
</script>

<Header title="Marketplace" subtitle="Browse and import Skills and Agents from public repositories">
	<button
		onclick={handleSyncAll}
		disabled={repoLibrary.isSyncing}
		class="btn btn-secondary"
	>
		<RefreshCw class="w-4 h-4 mr-2 {repoLibrary.isSyncing ? 'animate-spin' : ''}" />
		{repoLibrary.isSyncing ? 'Syncing...' : 'Sync All'}
	</button>
</Header>

<div class="flex-1 overflow-auto p-6">
	<!-- Rate Limit Info -->
	{#if repoLibrary.rateLimitInfo}
		<div class="mb-4 flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
			<Github class="w-4 h-4" />
			<span>
				GitHub API: {repoLibrary.rateLimitInfo.remaining}/{repoLibrary.rateLimitInfo.limit} requests remaining
			</span>
		</div>
	{/if}

	<!-- Tabs -->
	<div class="flex border-b border-gray-200 dark:border-gray-700 mb-6">
		<button
			onclick={() => (activeTab = 'mcps')}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
			'mcps'
				? 'border-primary-500 text-primary-600 dark:text-primary-400'
				: 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Server class="w-4 h-4" />
			MCPs ({repoLibrary.registryMcps.length})
		</button>
		<button
			onclick={() => (activeTab = 'skills')}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
			'skills'
				? 'border-primary-500 text-primary-600 dark:text-primary-400'
				: 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<FileCode class="w-4 h-4" />
			Skills ({getFilteredItems('skill').length})
		</button>
		<button
			onclick={() => (activeTab = 'agents')}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
			'agents'
				? 'border-primary-500 text-primary-600 dark:text-primary-400'
				: 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Bot class="w-4 h-4" />
			Agents ({getFilteredItems('subagent').length})
		</button>
		<button
			onclick={() => (activeTab = 'repos')}
			class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab ===
			'repos'
				? 'border-primary-500 text-primary-600 dark:text-primary-400'
				: 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}"
		>
			<Github class="w-4 h-4" />
			Repos ({repoLibrary.repos.length})
		</button>
	</div>

	<!-- Content -->
	{#if activeTab === 'mcps'}
		<!-- MCP Registry -->
		<div class="space-y-4">
			<!-- Search Bar -->
			<div class="flex gap-4">
				<div class="flex-1 relative">
					<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
					<input
						type="text"
						bind:value={mcpSearchQuery}
						onkeydown={handleMcpSearchKeydown}
						placeholder="Search MCP servers (e.g., filesystem, github, slack...)"
						class="input w-full pl-10"
					/>
				</div>
				<button
					onclick={handleMcpSearch}
					disabled={repoLibrary.isSearchingRegistry}
					class="btn btn-primary"
				>
					{#if repoLibrary.isSearchingRegistry}
						<Loader2 class="w-4 h-4 mr-2 animate-spin" />
						Searching...
					{:else}
						<Search class="w-4 h-4 mr-2" />
						Search
					{/if}
				</button>
			</div>

			<!-- Registry Error -->
			{#if repoLibrary.registryError}
				<div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-4 rounded-lg text-sm">
					{repoLibrary.registryError}
				</div>
			{/if}

			<!-- Loading State -->
			{#if repoLibrary.isSearchingRegistry && repoLibrary.registryMcps.length === 0}
				<div class="text-center py-12">
					<Loader2 class="w-8 h-8 mx-auto text-primary-500 animate-spin mb-4" />
					<p class="text-gray-500 dark:text-gray-400">Loading MCPs from registry...</p>
				</div>
			{:else if repoLibrary.registryMcps.length === 0}
				<div class="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
					<Server class="w-12 h-12 mx-auto text-gray-400 mb-4" />
					<p class="text-gray-500 dark:text-gray-400">No MCPs found</p>
					<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
						Try searching for an MCP server or wait for the registry to load
					</p>
				</div>
			{:else}
				<!-- MCP Grid -->
				<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
					{#each repoLibrary.registryMcps as mcp (mcp.registryId)}
						<button
							onclick={() => (selectedRegistryMcp = mcp)}
							class="card p-4 flex flex-col text-left hover:ring-2 hover:ring-primary-500/50 transition-all cursor-pointer"
						>
							<div class="flex items-start gap-3 mb-3">
								<div class="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900/50 flex items-center justify-center flex-shrink-0">
									<Server class="w-5 h-5 text-purple-600 dark:text-purple-400" />
								</div>
								<div class="flex-1 min-w-0">
									<h3 class="font-medium text-gray-900 dark:text-white truncate">{mcp.name}</h3>
									{#if mcp.description}
										<p class="text-sm text-gray-500 dark:text-gray-400 line-clamp-2">{mcp.description}</p>
									{/if}
								</div>
							</div>
							<div class="mt-auto flex items-center justify-between pt-3 border-t border-gray-100 dark:border-gray-700">
								<div class="flex items-center gap-2">
									{#if mcp.registryType}
										<span class="px-2 py-0.5 text-xs rounded {getMcpTypeColor(mcp.registryType)}">
											{mcp.registryType}
										</span>
									{/if}
									<span class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded">
										{mcp.mcpType}
									</span>
								</div>
								<span class="text-xs text-gray-400 flex items-center gap-1">
									<Eye class="w-3 h-3" />
									Click to preview
								</span>
							</div>
						</button>
					{/each}
				</div>

				<!-- Load More -->
				{#if repoLibrary.registryNextCursor}
					<div class="text-center pt-4">
						<button
							onclick={handleLoadMoreMcps}
							disabled={repoLibrary.isSearchingRegistry}
							class="btn btn-secondary"
						>
							{#if repoLibrary.isSearchingRegistry}
								<Loader2 class="w-4 h-4 mr-2 animate-spin" />
								Loading...
							{:else}
								Load More
							{/if}
						</button>
					</div>
				{/if}
			{/if}
		</div>
	{:else if activeTab === 'repos'}
		<!-- Repositories List -->
		<div class="space-y-4">
			<div class="flex justify-end gap-2">
				<button onclick={handleResetRepos} class="btn btn-secondary" title="Reset to default repos">
					<RotateCcw class="w-4 h-4 mr-2" />
					Reset Defaults
				</button>
				<button onclick={() => (showAddRepoModal = true)} class="btn btn-primary">
					<Plus class="w-4 h-4 mr-2" />
					Add Repository
				</button>
			</div>

			{#if repoLibrary.repos.length === 0}
				<div class="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
					<Store class="w-12 h-12 mx-auto text-gray-400 mb-4" />
					<p class="text-gray-500 dark:text-gray-400">No repositories configured</p>
					<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
						Add a repository to start browsing
					</p>
				</div>
			{:else}
				<div class="grid gap-4">
					{#each repoLibrary.repos as repo (repo.id)}
						<div
							class="card flex items-center justify-between p-4 {!repo.isEnabled
								? 'opacity-50'
								: ''}"
						>
							<div class="flex items-center gap-4">
								<div
									class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center"
								>
									<Github class="w-5 h-5 text-gray-600 dark:text-gray-400" />
								</div>
								<div>
									<div class="flex items-center gap-2">
										<h3 class="font-medium text-gray-900 dark:text-white">{repo.name}</h3>
										{#if repo.isDefault}
											<span
												class="px-2 py-0.5 text-xs bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300 rounded"
											>
												Default
											</span>
										{/if}
										<span
											class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded"
										>
											{repo.repoType === 'file_based' ? 'Files' : 'README'}
										</span>
										<span
											class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded"
										>
											{repo.contentType}
										</span>
									</div>
									{#if repo.description}
										<p class="text-sm text-gray-500 dark:text-gray-400">{repo.description}</p>
									{/if}
									{#if repo.lastFetchedAt}
										<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
											Last synced: {new Date(repo.lastFetchedAt).toLocaleString()}
										</p>
									{/if}
								</div>
							</div>
							<div class="flex items-center gap-2">
								<!-- Toggle -->
								<button
									onclick={() => handleToggleRepo(repo)}
									class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {repo.isEnabled
										? 'bg-primary-600'
										: 'bg-gray-300 dark:bg-gray-600'}"
									role="switch"
									aria-checked={repo.isEnabled}
									title={repo.isEnabled ? 'Disable' : 'Enable'}
								>
									<span
										class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {repo.isEnabled
											? 'translate-x-4'
											: 'translate-x-0'}"
									></span>
								</button>
								<!-- Sync -->
								<button
									onclick={() => handleSyncRepo(repo.id)}
									disabled={repoLibrary.isSyncing}
									class="p-2 text-gray-400 hover:text-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/20 rounded-lg transition-colors"
									title="Sync repository"
								>
									<RefreshCw class="w-4 h-4 {repoLibrary.isSyncing ? 'animate-spin' : ''}" />
								</button>
								<!-- Open in GitHub -->
								<a
									href={repo.githubUrl}
									target="_blank"
									rel="noopener noreferrer"
									class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg transition-colors"
									title="Open in GitHub"
								>
									<ExternalLink class="w-4 h-4" />
								</a>
								<!-- Remove -->
								{#if !repo.isDefault}
									<button
										onclick={() => handleRemoveRepo(repo)}
										class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
										title="Remove repository"
									>
										<Trash2 class="w-4 h-4" />
									</button>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{:else}
		<!-- Items Grid -->
		{@const items = activeTab === 'skills' ? getFilteredItems('skill') : getFilteredItems('subagent')}
		{@const itemType = activeTab === 'skills' ? 'skill' : 'subagent'}

		{#if items.length === 0}
			{@const EmptyIcon = typeIcons[itemType]}
			<div class="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
				<EmptyIcon class="w-12 h-12 mx-auto text-gray-400 mb-4" />
				<p class="text-gray-500 dark:text-gray-400">No {activeTab} found</p>
				<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
					Sync repositories to discover items
				</p>
			</div>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
				{#each items as item (item.id)}
					{@const IconComponent = typeIcons[item.itemType]}
					<button
						onclick={() => (selectedItem = item)}
						class="card p-4 flex flex-col text-left hover:ring-2 hover:ring-primary-500/50 transition-all cursor-pointer"
					>
						<div class="flex items-start gap-3 mb-3">
							<div class="w-10 h-10 rounded-lg {typeColors[item.itemType]} flex items-center justify-center flex-shrink-0">
								<IconComponent class="w-5 h-5" />
							</div>
							<div class="flex-1 min-w-0">
								<div class="flex items-center gap-2">
									<h3 class="font-medium text-gray-900 dark:text-white truncate">{item.name}</h3>
									{#if item.isImported}
										<span class="px-2 py-0.5 text-xs bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300 rounded">
											Imported
										</span>
									{/if}
								</div>
								{#if item.description}
									<p class="text-sm text-gray-500 dark:text-gray-400 line-clamp-2">{item.description}</p>
								{/if}
							</div>
						</div>
						<div class="mt-auto flex items-center justify-between pt-3 border-t border-gray-100 dark:border-gray-700">
							<span class="text-xs text-gray-400 flex items-center gap-1">
								<Eye class="w-3 h-3" />
								Click to preview
							</span>
							{#if item.isImported}
								<span class="px-2 py-1 text-xs bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300 rounded">
									Imported
								</span>
							{/if}
						</div>
					</button>
				{/each}
			</div>
		{/if}
	{/if}
</div>

<!-- Add Repository Modal -->
{#if showAddRepoModal}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={() => (showAddRepoModal = false)}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 p-6"
			onclick={(e) => e.stopPropagation()}
		>
			<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Add Repository</h2>

			<div class="space-y-4">
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						GitHub URL
					</label>
					<input
						type="url"
						bind:value={newRepoUrl}
						placeholder="https://github.com/owner/repo"
						class="input w-full"
					/>
				</div>

				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Repository Type
					</label>
					<select bind:value={newRepoType} class="input w-full">
						<option value="readme_based">README-based (parses README for links)</option>
						<option value="file_based">File-based (scans for .md files)</option>
					</select>
				</div>

				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Content Type
					</label>
					<select bind:value={newRepoContentType} class="input w-full">
						<option value="mixed">Mixed (Skills & Agents)</option>
						<option value="skill">Skills only</option>
						<option value="subagent">Agents only</option>
					</select>
				</div>
			</div>

			<div class="mt-6 flex justify-end gap-3">
				<button onclick={() => (showAddRepoModal = false)} class="btn btn-secondary">
					Cancel
				</button>
				<button onclick={handleAddRepo} disabled={!newRepoUrl} class="btn btn-primary">
					Add Repository
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Item Preview Modal -->
{#if selectedItem}
	{@const ModalIconComponent = typeIcons[selectedItem.itemType]}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={() => (selectedItem = null)}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[85vh] flex flex-col"
			onclick={(e) => e.stopPropagation()}
		>
			<!-- Header -->
			<div class="flex items-start justify-between p-6 border-b border-gray-200 dark:border-gray-700">
				<div class="flex items-start gap-4">
					<div class="w-12 h-12 rounded-lg {typeColors[selectedItem.itemType]} flex items-center justify-center flex-shrink-0">
						<ModalIconComponent class="w-6 h-6" />
					</div>
					<div>
						<div class="flex items-center gap-2">
							<h2 class="text-xl font-semibold text-gray-900 dark:text-white">{selectedItem.name}</h2>
							<span class="px-2 py-0.5 text-xs {typeColors[selectedItem.itemType]} rounded">
								{selectedItem.itemType}
							</span>
							{#if selectedItem.isImported}
								<span class="px-2 py-0.5 text-xs bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300 rounded">
									Imported
								</span>
							{/if}
						</div>
						{#if selectedItem.description}
							<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{selectedItem.description}</p>
						{/if}
					</div>
				</div>
				<button
					onclick={() => (selectedItem = null)}
					class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg transition-colors"
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-auto p-6">
				{#if selectedItem.rawContent}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Content Preview</h3>
						<pre class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 text-sm text-gray-800 dark:text-gray-200 overflow-x-auto whitespace-pre-wrap font-mono max-h-64 overflow-y-auto">{selectedItem.rawContent}</pre>
					</div>
				{/if}

				{#if selectedItem.metadata}
					{@const metadata = JSON.parse(selectedItem.metadata)}
					{#if Object.keys(metadata).length > 0}
						<div class="mb-4">
							<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Metadata</h3>
							<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
								<dl class="grid grid-cols-2 gap-2 text-sm">
									{#each Object.entries(metadata) as [key, value]}
										<dt class="text-gray-500 dark:text-gray-400 capitalize">{key.replace(/-/g, ' ')}</dt>
										<dd class="text-gray-900 dark:text-white">{value}</dd>
									{/each}
								</dl>
							</div>
						</div>
					{/if}
				{/if}

				{#if selectedItem.filePath}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">File Path</h3>
						<code class="text-sm text-gray-600 dark:text-gray-400">{selectedItem.filePath}</code>
					</div>
				{/if}

				{#if selectedItem.sourceUrl}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Source</h3>
						<a
							href={selectedItem.sourceUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="text-sm text-primary-600 hover:text-primary-700 dark:text-primary-400 flex items-center gap-1"
						>
							<ExternalLink class="w-4 h-4" />
							{selectedItem.sourceUrl}
						</a>
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-end gap-3 p-6 border-t border-gray-200 dark:border-gray-700">
				<button onclick={() => (selectedItem = null)} class="btn btn-secondary">
					Close
				</button>
				<button
					onclick={() => {
						handleImport(selectedItem!);
						selectedItem = null;
					}}
					disabled={selectedItem.isImported}
					class="btn {selectedItem.isImported ? 'btn-ghost text-gray-400' : 'btn-primary'}"
				>
					<Download class="w-4 h-4 mr-2" />
					{selectedItem.isImported ? 'Already Imported' : 'Import'}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- MCP Registry Preview Modal -->
{#if selectedRegistryMcp}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		onclick={() => (selectedRegistryMcp = null)}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[85vh] flex flex-col"
			onclick={(e) => e.stopPropagation()}
		>
			<!-- Header -->
			<div class="flex items-start justify-between p-6 border-b border-gray-200 dark:border-gray-700">
				<div class="flex items-start gap-4">
					<div class="w-12 h-12 rounded-lg bg-purple-100 dark:bg-purple-900/50 flex items-center justify-center flex-shrink-0">
						<Server class="w-6 h-6 text-purple-600 dark:text-purple-400" />
					</div>
					<div>
						<div class="flex items-center gap-2 flex-wrap">
							<h2 class="text-xl font-semibold text-gray-900 dark:text-white">{selectedRegistryMcp.name}</h2>
							{#if selectedRegistryMcp.registryType}
								<span class="px-2 py-0.5 text-xs rounded {getMcpTypeColor(selectedRegistryMcp.registryType)}">
									{selectedRegistryMcp.registryType}
								</span>
							{/if}
							<span class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded">
								{selectedRegistryMcp.mcpType}
							</span>
							{#if selectedRegistryMcp.version}
								<span class="px-2 py-0.5 text-xs bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400 rounded">
									v{selectedRegistryMcp.version}
								</span>
							{/if}
						</div>
						{#if selectedRegistryMcp.description}
							<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{selectedRegistryMcp.description}</p>
						{/if}
					</div>
				</div>
				<button
					onclick={() => (selectedRegistryMcp = null)}
					class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg transition-colors"
				>
					<X class="w-5 h-5" />
				</button>
			</div>

			<!-- Content -->
			<div class="flex-1 overflow-auto p-6">
				<!-- Command/URL Configuration -->
				<div class="mb-4">
					<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Configuration</h3>
					<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
						{#if selectedRegistryMcp.mcpType === 'stdio'}
							<div class="space-y-2">
								<div>
									<span class="text-xs text-gray-500 dark:text-gray-400">Command:</span>
									<code class="ml-2 text-sm text-gray-800 dark:text-gray-200 font-mono">{selectedRegistryMcp.command}</code>
								</div>
								{#if selectedRegistryMcp.args && selectedRegistryMcp.args.length > 0}
									<div>
										<span class="text-xs text-gray-500 dark:text-gray-400">Arguments:</span>
										<code class="ml-2 text-sm text-gray-800 dark:text-gray-200 font-mono">{selectedRegistryMcp.args.join(' ')}</code>
									</div>
								{/if}
							</div>
						{:else}
							<div>
								<span class="text-xs text-gray-500 dark:text-gray-400">URL:</span>
								<code class="ml-2 text-sm text-gray-800 dark:text-gray-200 font-mono break-all">{selectedRegistryMcp.url}</code>
							</div>
						{/if}
					</div>
				</div>

				<!-- Environment Variables -->
				{#if selectedRegistryMcp.envPlaceholders && selectedRegistryMcp.envPlaceholders.length > 0}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							Environment Variables
							<span class="text-xs text-gray-400 font-normal ml-2">(You'll need to configure these after import)</span>
						</h3>
						<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 space-y-3">
							{#each selectedRegistryMcp.envPlaceholders as envVar}
								<div class="flex items-start gap-2">
									<code class="text-sm font-mono text-gray-800 dark:text-gray-200 bg-gray-200 dark:bg-gray-700 px-2 py-0.5 rounded">
										{envVar.name}
									</code>
									{#if envVar.isRequired}
										<span class="px-1.5 py-0.5 text-xs bg-red-100 text-red-600 dark:bg-red-900/50 dark:text-red-400 rounded">
											Required
										</span>
									{/if}
									{#if envVar.description}
										<span class="text-sm text-gray-500 dark:text-gray-400">{envVar.description}</span>
									{/if}
								</div>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Source URL -->
				{#if selectedRegistryMcp.sourceUrl}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Source Repository</h3>
						<a
							href={selectedRegistryMcp.sourceUrl}
							target="_blank"
							rel="noopener noreferrer"
							class="text-sm text-primary-600 hover:text-primary-700 dark:text-primary-400 flex items-center gap-1"
						>
							<ExternalLink class="w-4 h-4" />
							{selectedRegistryMcp.sourceUrl}
						</a>
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="flex items-center justify-end gap-3 p-6 border-t border-gray-200 dark:border-gray-700">
				<button onclick={() => (selectedRegistryMcp = null)} class="btn btn-secondary">
					Close
				</button>
				<button
					onclick={() => handleImportRegistryMcp(selectedRegistryMcp!)}
					disabled={isImportingMcp}
					class="btn btn-primary"
				>
					{#if isImportingMcp}
						<Loader2 class="w-4 h-4 mr-2 animate-spin" />
						Importing...
					{:else}
						<Download class="w-4 h-4 mr-2" />
						Import to Library
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}
