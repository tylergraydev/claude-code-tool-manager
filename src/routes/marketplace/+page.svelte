<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
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
		Loader2,
		ArrowUpDown
	} from 'lucide-svelte';
	import { i18n } from '$lib/i18n';
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
	let mcpSortBy = $state<'name' | 'updated'>('updated');

	// Sorted MCPs
	const sortedRegistryMcps = $derived.by(() => {
		const mcps = [...repoLibrary.registryMcps];
		if (mcpSortBy === 'updated') {
			return mcps.sort((a, b) => {
				const dateA = a.updatedAt ? new Date(a.updatedAt).getTime() : 0;
				const dateB = b.updatedAt ? new Date(b.updatedAt).getTime() : 0;
				return dateB - dateA; // Descending (most recent first)
			});
		} else {
			return mcps.sort((a, b) => a.name.localeCompare(b.name));
		}
	});

	// Load data on mount
	onMount(() => {
		loadData();
		// Load MCP Registry immediately since MCPs is the default tab
		repoLibrary.loadRegistryMcps();
	});

	async function loadData() {
		await repoLibrary.loadRepos();
		await repoLibrary.loadItems();
		repoLibrary.checkRateLimit();

		// Auto-sync if no items have been fetched yet
		if (repoLibrary.repos.length > 0 && repoLibrary.items.length === 0) {
			notifications.info(i18n.t('marketplace.syncing'));
			await handleSyncAll();
		}
	}

	async function handleSyncAll() {
		try {
			const result = await repoLibrary.syncAllRepos();
			if (result.errors.length > 0) {
				notifications.warning(i18n.t('marketplace.syncErrors', { count: result.errors.length }));
			} else {
				notifications.success(i18n.t('marketplace.syncResult', { added: result.added, updated: result.updated }));
			}
		} catch (e) {
			notifications.error(i18n.t('marketplace.syncFailed'));
		}
	}

	async function handleSyncRepo(id: number) {
		try {
			const result = await repoLibrary.syncRepo(id);
			notifications.success(i18n.t('marketplace.syncResult', { added: result.added, updated: result.updated }));
		} catch (e) {
			notifications.error(i18n.t('marketplace.syncRepoFailed'));
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
			notifications.success(i18n.t('marketplace.repoAdded'));
			showAddRepoModal = false;
			newRepoUrl = '';
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleRemoveRepo(repo: Repo) {
		if (repo.isDefault) {
			notifications.error(i18n.t('marketplace.resetFailed'));
			return;
		}
		try {
			await repoLibrary.removeRepo(repo.id);
			notifications.success(i18n.t('marketplace.repoRemoved'));
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleToggleRepo(repo: Repo) {
		try {
			await repoLibrary.toggleRepo(repo.id, !repo.isEnabled);
		} catch (e) {
			notifications.error(i18n.t('marketplace.toggleFailed'));
		}
	}

	async function handleImport(item: RepoItem) {
		try {
			const result = await repoLibrary.importItem(item.id);
			if (result.success) {
				notifications.success(i18n.t('marketplace.itemImported', { name: item.name }));
			} else {
				notifications.warning(result.message || i18n.t('marketplace.alreadyImportedMsg'));
			}
		} catch (e) {
			notifications.error(i18n.t('marketplace.importFailed'));
		}
	}

	async function handleResetRepos() {
		try {
			await invoke('reset_repos_to_defaults');
			await repoLibrary.loadRepos();
			await repoLibrary.loadItems();
			notifications.success(i18n.t('marketplace.resetDone'));
			await handleSyncAll();
		} catch (e) {
			notifications.error(i18n.t('marketplace.resetFailed'));
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
			notifications.success(i18n.t('marketplace.mcpImported', { name: entry.name }));
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

<Header title={i18n.t('page.marketplace.title')} subtitle={i18n.t('page.marketplace.subtitle')}>
	<button
		onclick={handleSyncAll}
		disabled={repoLibrary.isSyncing}
		class="btn btn-secondary"
	>
		<RefreshCw class="w-4 h-4 mr-2 {repoLibrary.isSyncing ? 'animate-spin' : ''}" />
		{repoLibrary.isSyncing ? i18n.t('marketplace.syncing') : i18n.t('marketplace.syncAll')}
	</button>
</Header>

<div class="flex-1 overflow-auto p-6">
	<!-- Rate Limit Info Bar -->
	{#if repoLibrary.rateLimitInfo}
		{@const info = repoLibrary.rateLimitInfo}
		{@const isAuthenticated = info.limit > 60}
		{@const isLow = info.remaining < 10}
		<div class="mb-4 flex items-center justify-between px-4 py-2.5 rounded-lg text-sm {isLow ? 'bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800' : 'bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700'}">
			<div class="flex items-center gap-3">
				<span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium {isAuthenticated ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'}">
					{isAuthenticated ? i18n.t('marketplace.authenticated') : i18n.t('marketplace.unauthenticated')}
				</span>
				<span class="{isLow ? 'text-amber-700 dark:text-amber-400' : 'text-gray-600 dark:text-gray-400'}">
					{i18n.t('marketplace.apiRateInfo', { remaining: info.remaining, limit: info.limit })}
				</span>
			</div>
			{#if isLow && info.resetAt}
				<span class="text-xs text-amber-600 dark:text-amber-400">
					{i18n.t('marketplace.resetsAt', { time: new Date(info.resetAt).toLocaleTimeString() })}
				</span>
			{/if}
			{#if !isAuthenticated}
				<a href="/settings" class="text-xs text-primary-600 dark:text-primary-400 hover:underline">
					{i18n.t('marketplace.addToken')}
				</a>
			{/if}
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
						placeholder={i18n.t('marketplace.searchPlaceholder')}
						class="input w-full pl-10"
					/>
				</div>
				<!-- Sort Dropdown -->
				<div class="relative">
					<select
						bind:value={mcpSortBy}
						class="input pr-8 appearance-none cursor-pointer"
					>
						<option value="updated">{i18n.t('marketplace.recentlyUpdated')}</option>
						<option value="name">{i18n.t('marketplace.nameAZ')}</option>
					</select>
					<ArrowUpDown class="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" />
				</div>
				<button
					onclick={handleMcpSearch}
					disabled={repoLibrary.isSearchingRegistry}
					class="btn btn-primary"
				>
					{#if repoLibrary.isSearchingRegistry}
						<Loader2 class="w-4 h-4 mr-2 animate-spin" />
						{i18n.t('marketplace.searching')}
					{:else}
						<Search class="w-4 h-4 mr-2" />
						{i18n.t('common.search')}
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
					<p class="text-gray-500 dark:text-gray-400">{i18n.t('marketplace.loadingMcps')}</p>
				</div>
			{:else if repoLibrary.registryMcps.length === 0}
				<div class="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
					<Server class="w-12 h-12 mx-auto text-gray-400 mb-4" />
					<p class="text-gray-500 dark:text-gray-400">{i18n.t('marketplace.noMcpsFound')}</p>
					<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
						{i18n.t('marketplace.noMcpsHint')}
					</p>
				</div>
			{:else}
				<!-- MCP Grid -->
				<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
					{#each sortedRegistryMcps as mcp, i (`${mcp.registryId}-${i}`)}
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
									{i18n.t('marketplace.clickToPreview')}
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
								{i18n.t('common.loading')}
							{:else}
								{i18n.t('marketplace.loadMore')}
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
				<button onclick={handleResetRepos} class="btn btn-secondary" title={i18n.t('marketplace.resetDefaults')}>
					<RotateCcw class="w-4 h-4 mr-2" />
					{i18n.t('marketplace.resetDefaults')}
				</button>
				<button onclick={() => (showAddRepoModal = true)} class="btn btn-primary">
					<Plus class="w-4 h-4 mr-2" />
					{i18n.t('marketplace.addRepository')}
				</button>
			</div>

			{#if repoLibrary.repos.length === 0}
				<div class="text-center py-12 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
					<Store class="w-12 h-12 mx-auto text-gray-400 mb-4" />
					<p class="text-gray-500 dark:text-gray-400">{i18n.t('marketplace.noRepos')}</p>
					<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
						{i18n.t('marketplace.addRepoHint')}
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
												{i18n.t('common.default')}
											</span>
										{/if}
										<span
											class="px-2 py-0.5 text-xs bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400 rounded"
										>
											{repo.repoType === 'file_based' ? i18n.t('marketplace.files') : i18n.t('marketplace.readme')}
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
											{i18n.t('marketplace.lastSynced', { date: new Date(repo.lastFetchedAt).toLocaleString() })}
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
									title={repo.isEnabled ? i18n.t('common.disable') : i18n.t('common.enable')}
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
									title={i18n.t('marketplace.syncRepo')}
								>
									<RefreshCw class="w-4 h-4 {repoLibrary.isSyncing ? 'animate-spin' : ''}" />
								</button>
								<!-- Open in GitHub -->
								<a
									href={repo.githubUrl}
									target="_blank"
									rel="noopener noreferrer"
									class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg transition-colors"
									title={i18n.t('marketplace.openInGithub')}
								>
									<ExternalLink class="w-4 h-4" />
								</a>
								<!-- Remove -->
								{#if !repo.isDefault}
									<button
										onclick={() => handleRemoveRepo(repo)}
										class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
										title={i18n.t('marketplace.removeRepo')}
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
				<p class="text-gray-500 dark:text-gray-400">{i18n.t('marketplace.noItemsFound', { type: activeTab })}</p>
				<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
					{i18n.t('marketplace.syncToDiscover')}
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
											{i18n.t('marketplace.imported')}
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
								{i18n.t('marketplace.clickToPreview')}
							</span>
							{#if item.isImported}
								<span class="px-2 py-1 text-xs bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300 rounded">
									{i18n.t('marketplace.imported')}
								</span>
							{/if}
						</div>
					</button>
				{/each}
			</div>
		{/if}
	{/if}
</div>

<!-- {i18n.t('marketplace.addRepository')} Modal -->
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
			<h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">{i18n.t('marketplace.addRepository')}</h2>

			<div class="space-y-4">
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{i18n.t('marketplace.githubUrl')}
					</label>
					<input
						type="url"
						bind:value={newRepoUrl}
						placeholder={i18n.t('marketplace.urlPlaceholder')}
						class="input w-full"
					/>
				</div>

				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{i18n.t('marketplace.repoType')}
					</label>
					<select bind:value={newRepoType} class="input w-full">
						<option value="readme_based">{i18n.t('marketplace.readmeBased')}</option>
						<option value="file_based">{i18n.t('marketplace.fileBased')}</option>
					</select>
				</div>

				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						{i18n.t('marketplace.contentType')}
					</label>
					<select bind:value={newRepoContentType} class="input w-full">
						<option value="mixed">{i18n.t('marketplace.mixed')}</option>
						<option value="skill">{i18n.t('marketplace.skillsOnly')}</option>
						<option value="subagent">{i18n.t('marketplace.agentsOnly')}</option>
					</select>
				</div>
			</div>

			<div class="mt-6 flex justify-end gap-3">
				<button onclick={() => (showAddRepoModal = false)} class="btn btn-secondary">
					{i18n.t('common.cancel')}
				</button>
				<button onclick={handleAddRepo} disabled={!newRepoUrl} class="btn btn-primary">
					{i18n.t('marketplace.addRepository')}
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
									{i18n.t('marketplace.imported')}
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
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{i18n.t('marketplace.contentPreview')}</h3>
						<pre class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 text-sm text-gray-800 dark:text-gray-200 overflow-x-auto whitespace-pre-wrap font-mono max-h-64 overflow-y-auto">{selectedItem.rawContent}</pre>
					</div>
				{/if}

				{#if selectedItem.metadata}
					{@const metadata = JSON.parse(selectedItem.metadata)}
					{#if Object.keys(metadata).length > 0}
						<div class="mb-4">
							<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{i18n.t('marketplace.metadata')}</h3>
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
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{i18n.t('marketplace.filePath')}</h3>
						<code class="text-sm text-gray-600 dark:text-gray-400">{selectedItem.filePath}</code>
					</div>
				{/if}

				{#if selectedItem.sourceUrl}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{i18n.t('marketplace.source')}</h3>
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
					{i18n.t('common.close')}
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
					{selectedItem.isImported ? i18n.t('marketplace.alreadyImported') : i18n.t('common.import')}
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
					<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{i18n.t('marketplace.configuration')}</h3>
					<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
						{#if selectedRegistryMcp.mcpType === 'stdio'}
							<div class="space-y-2">
								<div>
									<span class="text-xs text-gray-500 dark:text-gray-400">{i18n.t('marketplace.commandLabel')}</span>
									<code class="ml-2 text-sm text-gray-800 dark:text-gray-200 font-mono">{selectedRegistryMcp.command}</code>
								</div>
								{#if selectedRegistryMcp.args && selectedRegistryMcp.args.length > 0}
									<div>
										<span class="text-xs text-gray-500 dark:text-gray-400">{i18n.t('marketplace.arguments')}</span>
										<code class="ml-2 text-sm text-gray-800 dark:text-gray-200 font-mono">{selectedRegistryMcp.args.join(' ')}</code>
									</div>
								{/if}
							</div>
						{:else}
							<div>
								<span class="text-xs text-gray-500 dark:text-gray-400">{i18n.t('marketplace.urlLabel')}</span>
								<code class="ml-2 text-sm text-gray-800 dark:text-gray-200 font-mono break-all">{selectedRegistryMcp.url}</code>
							</div>
						{/if}
					</div>
				</div>

				<!-- Environment Variables -->
				{#if selectedRegistryMcp.envPlaceholders && selectedRegistryMcp.envPlaceholders.length > 0}
					<div class="mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
							{i18n.t('marketplace.envVars')}
							<span class="text-xs text-gray-400 font-normal ml-2">{i18n.t('marketplace.envVarsHint')}</span>
						</h3>
						<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4 space-y-3">
							{#each selectedRegistryMcp.envPlaceholders as envVar}
								<div class="flex items-start gap-2">
									<code class="text-sm font-mono text-gray-800 dark:text-gray-200 bg-gray-200 dark:bg-gray-700 px-2 py-0.5 rounded">
										{envVar.name}
									</code>
									{#if envVar.isRequired}
										<span class="px-1.5 py-0.5 text-xs bg-red-100 text-red-600 dark:bg-red-900/50 dark:text-red-400 rounded">
											{i18n.t('marketplace.required')}
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
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{i18n.t('marketplace.sourceRepo')}</h3>
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
					{i18n.t('common.close')}
				</button>
				<button
					onclick={() => handleImportRegistryMcp(selectedRegistryMcp!)}
					disabled={isImportingMcp}
					class="btn btn-primary"
				>
					{#if isImportingMcp}
						<Loader2 class="w-4 h-4 mr-2 animate-spin" />
						{i18n.t('marketplace.importing')}
					{:else}
						<Download class="w-4 h-4 mr-2" />
						{i18n.t('marketplace.importToLibrary')}
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}
