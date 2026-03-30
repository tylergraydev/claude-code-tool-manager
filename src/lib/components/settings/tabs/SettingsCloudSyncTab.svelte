<script lang="ts">
	import { onMount } from 'svelte';
	import { cloudSyncStore, projectsStore, notifications } from '$lib/stores';
	import { Cloud, Github, Check, AlertCircle, Upload, Download, ExternalLink, LogOut, Loader2, FolderSync, RefreshCw } from 'lucide-svelte';
	import type { SyncConfig, ProjectMapping } from '$lib/types';

	let isInitialized = $state(false);

	onMount(async () => {
		await Promise.all([
			cloudSyncStore.load(),
			projectsStore.loadProjects()
		]);
		isInitialized = true;
	});

	// Local config for editing before save
	let localConfig = $derived(cloudSyncStore.syncConfig ?? {
		syncGlobalClaudeMd: true,
		syncSkills: false,
		syncMcps: false,
		syncProjectClaudeMds: [] as string[],
		autoSyncOnLaunch: false
	});

	async function handleConnect() {
		try {
			await cloudSyncStore.connect();
			notifications.success(`Connected as ${cloudSyncStore.authStatus?.username}`);
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleDisconnect() {
		await cloudSyncStore.disconnect();
		notifications.success('Cloud sync disconnected');
	}

	async function toggleConfigOption(key: keyof SyncConfig, value: boolean) {
		const newConfig = { ...localConfig, [key]: value } as SyncConfig;
		await cloudSyncStore.saveConfig(newConfig);
	}

	async function toggleProjectSync(projectId: string, enabled: boolean) {
		const current = [...localConfig.syncProjectClaudeMds];
		const newList = enabled
			? [...current, projectId]
			: current.filter(id => id !== projectId);
		const newConfig = { ...localConfig, syncProjectClaudeMds: newList } as SyncConfig;
		await cloudSyncStore.saveConfig(newConfig);

		// Auto-create mapping for newly enabled projects
		if (enabled) {
			const project = projectsStore.projects.find(p => String(p.id) === projectId);
			if (project) {
				const existing = cloudSyncStore.projectMappings;
				const canonical = project.path.split('/').pop() ?? project.name;
				if (!existing.find(m => m.localPath === project.path)) {
					const newMappings: ProjectMapping[] = [...existing, { localPath: project.path, canonicalName: canonical }];
					await cloudSyncStore.saveMappings(newMappings);
				}
			}
		}
	}

	async function updateMapping(index: number, canonicalName: string) {
		const updated = [...cloudSyncStore.projectMappings];
		updated[index] = { ...updated[index], canonicalName };
		await cloudSyncStore.saveMappings(updated);
	}

	async function handlePush() {
		const result = await cloudSyncStore.push();
		if (result) {
			if (result.pushed.length > 0) {
				notifications.success(`Pushed: ${result.pushed.join(', ')}`);
			} else {
				notifications.success('Push complete (nothing to sync)');
			}
			if (result.conflicts.length > 0) {
				notifications.error(`Conflicts: ${result.conflicts.join(', ')}`);
			}
		}
	}

	async function handlePull() {
		const result = await cloudSyncStore.pull();
		if (result) {
			if (result.pulled.length > 0) {
				notifications.success(`Pulled: ${result.pulled.join(', ')}`);
			} else {
				notifications.success('Pull complete (nothing new)');
			}
			if (result.conflicts.length > 0) {
				notifications.error(`Conflicts: ${result.conflicts.join(', ')}`);
			}
		}
	}

	function formatDate(iso: string | null): string {
		if (!iso) return 'Never';
		try {
			return new Date(iso).toLocaleString();
		} catch {
			return iso;
		}
	}
</script>

<div class="space-y-8">
	<!-- Authentication Card -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
			<Github class="w-5 h-5" />
			GitHub Connection
		</h3>
		<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
			Connect via the GitHub CLI to sync your Claude Code configuration across machines using a private Gist.
		</p>

		{#if !isInitialized || cloudSyncStore.isLoading}
			<div class="flex items-center gap-2 text-gray-400">
				<Loader2 class="w-4 h-4 animate-spin" />
				<span class="text-sm">Loading...</span>
			</div>
		{:else if cloudSyncStore.isAuthenticated}
			<div class="flex items-center justify-between p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
				<div class="flex items-center gap-3">
					<div class="w-10 h-10 rounded-full bg-green-500 text-white flex items-center justify-center font-bold text-lg">
						{cloudSyncStore.authStatus?.username?.charAt(0).toUpperCase() ?? 'G'}
					</div>
					<div>
						<p class="font-medium text-green-800 dark:text-green-200">
							{cloudSyncStore.authStatus?.username}
						</p>
						<div class="flex items-center gap-2 text-xs text-green-600 dark:text-green-400">
							<Check class="w-3 h-3" />
							Connected
							{#if cloudSyncStore.authStatus?.gistUrl}
								<span class="text-gray-400">|</span>
								<a
									href={cloudSyncStore.authStatus.gistUrl}
									target="_blank"
									rel="noopener noreferrer"
									class="hover:underline inline-flex items-center gap-1"
								>
									View Gist <ExternalLink class="w-3 h-3" />
								</a>
							{/if}
						</div>
					</div>
				</div>
				<button
					onclick={handleDisconnect}
					class="btn btn-secondary text-sm flex items-center gap-1.5 text-red-600 dark:text-red-400"
				>
					<LogOut class="w-4 h-4" />
					Disconnect
				</button>
			</div>
		{:else}
			<div class="space-y-3">
				{#if cloudSyncStore.authStatus?.hasGhCli === false}
					<div class="p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
						<p class="text-sm text-amber-700 dark:text-amber-400">
							<AlertCircle class="w-4 h-4 inline mr-1" />
							GitHub CLI not found. Install it from <a href="https://cli.github.com" target="_blank" rel="noopener noreferrer" class="underline">cli.github.com</a> and run <code class="px-1.5 py-0.5 bg-amber-100 dark:bg-amber-900/40 rounded text-xs">gh auth login</code>.
						</p>
					</div>
				{/if}

				<button
					onclick={handleConnect}
					disabled={cloudSyncStore.isConnecting}
					class="btn btn-primary flex items-center gap-2"
				>
					{#if cloudSyncStore.isConnecting}
						<Loader2 class="w-4 h-4 animate-spin" />
						Connecting...
					{:else}
						<Github class="w-4 h-4" />
						Connect with GitHub CLI
					{/if}
				</button>

				{#if cloudSyncStore.error}
					<div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
						<p class="text-sm text-red-700 dark:text-red-400">{cloudSyncStore.error}</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<!-- What to Sync Card -->
	{#if cloudSyncStore.isAuthenticated}
		<div class="card">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
				<Cloud class="w-5 h-5" />
				What to Sync
			</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
				Choose which configuration items to include in cloud sync.
			</p>

			<div class="space-y-3">
				<!-- Global CLAUDE.md -->
				<label class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-700 transition-colors cursor-pointer">
					<div>
						<p class="font-medium text-gray-900 dark:text-white">Global CLAUDE.md</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Your personal instructions file (~/.claude/CLAUDE.md)</p>
					</div>
					<input
						type="checkbox"
						checked={localConfig.syncGlobalClaudeMd}
						onchange={(e) => toggleConfigOption('syncGlobalClaudeMd', (e.target as HTMLInputElement).checked)}
						class="w-5 h-5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600"
					/>
				</label>

				<!-- Skills -->
				<label class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-700 transition-colors cursor-pointer">
					<div>
						<p class="font-medium text-gray-900 dark:text-white">Skills & Agents</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Custom skills, slash commands, and agent configurations</p>
					</div>
					<input
						type="checkbox"
						checked={localConfig.syncSkills}
						onchange={(e) => toggleConfigOption('syncSkills', (e.target as HTMLInputElement).checked)}
						class="w-5 h-5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600"
					/>
				</label>

				<!-- MCPs -->
				<label class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-700 transition-colors cursor-pointer">
					<div>
						<p class="font-medium text-gray-900 dark:text-white">MCP Servers</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">MCP server configurations (excluding system MCPs)</p>
					</div>
					<input
						type="checkbox"
						checked={localConfig.syncMcps}
						onchange={(e) => toggleConfigOption('syncMcps', (e.target as HTMLInputElement).checked)}
						class="w-5 h-5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600"
					/>
				</label>

				<!-- Auto-sync -->
				<label class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-700 transition-colors cursor-pointer">
					<div>
						<p class="font-medium text-gray-900 dark:text-white">Auto-sync on launch</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">Automatically pull latest config when the app starts</p>
					</div>
					<input
						type="checkbox"
						checked={localConfig.autoSyncOnLaunch}
						onchange={(e) => toggleConfigOption('autoSyncOnLaunch', (e.target as HTMLInputElement).checked)}
						class="w-5 h-5 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600"
					/>
				</label>
			</div>
		</div>

		<!-- Project CLAUDE.md Sync -->
		<div class="card">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
				<FolderSync class="w-5 h-5" />
				Project Sync
			</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
				Select projects whose CLAUDE.md files should be synced. Each project gets a canonical name used to match it across machines.
			</p>

			{#if projectsStore.projects.length === 0}
				<div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg text-center">
					<p class="text-sm text-gray-500 dark:text-gray-400">
						No projects configured. Add projects from the Projects page to sync them.
					</p>
				</div>
			{:else}
				<div class="space-y-2">
					{#each projectsStore.projects as project}
						{@const projectId = String(project.id)}
						{@const isEnabled = localConfig.syncProjectClaudeMds.includes(projectId)}
						{@const mapping = cloudSyncStore.projectMappings.find(m => m.localPath === project.path)}
						<div class="p-3 rounded-lg border border-gray-200 dark:border-gray-700 {isEnabled ? 'border-primary-500/50 bg-primary-50/50 dark:bg-primary-900/10' : ''}">
							<div class="flex items-center justify-between">
								<div class="flex-1 min-w-0">
									<div class="flex items-center gap-2">
										<label class="flex items-center gap-2 cursor-pointer">
											<input
												type="checkbox"
												checked={isEnabled}
												onchange={(e) => toggleProjectSync(projectId, (e.target as HTMLInputElement).checked)}
												class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:bg-gray-700 dark:border-gray-600"
											/>
											<span class="font-medium text-gray-900 dark:text-white text-sm">{project.name}</span>
										</label>
									</div>
									<p class="text-xs text-gray-500 dark:text-gray-400 ml-6 truncate">{project.path}</p>
								</div>
								{#if isEnabled && mapping}
									<div class="flex items-center gap-2 ml-4">
										<span class="text-xs text-gray-500 dark:text-gray-400">as</span>
										<input
											type="text"
											value={mapping.canonicalName}
											onchange={(e) => {
												const idx = cloudSyncStore.projectMappings.indexOf(mapping);
												if (idx >= 0) updateMapping(idx, (e.target as HTMLInputElement).value);
											}}
											class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-white w-32"
											placeholder="canonical name"
										/>
									</div>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Sync Controls Card -->
		<div class="card">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
				<RefreshCw class="w-5 h-5" />
				Sync Controls
			</h3>

			<div class="flex items-center gap-3 mb-6">
				<button
					onclick={handlePush}
					disabled={cloudSyncStore.isPushing || cloudSyncStore.isPulling}
					class="btn btn-primary flex items-center gap-2"
				>
					{#if cloudSyncStore.isPushing}
						<Loader2 class="w-4 h-4 animate-spin" />
						Pushing...
					{:else}
						<Upload class="w-4 h-4" />
						Push to Cloud
					{/if}
				</button>
				<button
					onclick={handlePull}
					disabled={cloudSyncStore.isPulling || cloudSyncStore.isPushing}
					class="btn btn-secondary flex items-center gap-2"
				>
					{#if cloudSyncStore.isPulling}
						<Loader2 class="w-4 h-4 animate-spin" />
						Pulling...
					{:else}
						<Download class="w-4 h-4" />
						Pull from Cloud
					{/if}
				</button>
			</div>

			<!-- Sync Status -->
			{#if cloudSyncStore.syncStatus}
				<div class="grid grid-cols-2 gap-4 text-sm">
					<div class="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<p class="text-xs text-gray-500 dark:text-gray-400 mb-1">Last Pushed</p>
						<p class="font-medium text-gray-900 dark:text-white">
							{formatDate(cloudSyncStore.syncStatus.lastPushedAt)}
						</p>
					</div>
					<div class="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<p class="text-xs text-gray-500 dark:text-gray-400 mb-1">Last Pulled</p>
						<p class="font-medium text-gray-900 dark:text-white">
							{formatDate(cloudSyncStore.syncStatus.lastPulledAt)}
						</p>
					</div>
				</div>

				{#if cloudSyncStore.syncStatus.itemCounts}
					<div class="mt-4 flex flex-wrap gap-2">
						{#if cloudSyncStore.syncStatus.itemCounts.hasGlobalClaudeMd}
							<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300">
								CLAUDE.md
							</span>
						{/if}
						{#if cloudSyncStore.syncStatus.itemCounts.mcps > 0}
							<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300">
								{cloudSyncStore.syncStatus.itemCounts.mcps} MCPs
							</span>
						{/if}
						{#if cloudSyncStore.syncStatus.itemCounts.skills > 0}
							<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-emerald-100 text-emerald-800 dark:bg-emerald-900/30 dark:text-emerald-300">
								{cloudSyncStore.syncStatus.itemCounts.skills} Skills
							</span>
						{/if}
						{#if cloudSyncStore.syncStatus.itemCounts.projects > 0}
							<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300">
								{cloudSyncStore.syncStatus.itemCounts.projects} Projects
							</span>
						{/if}
					</div>
				{/if}
			{/if}

			<!-- Last Result -->
			{#if cloudSyncStore.lastResult}
				<div class="mt-4 p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
					<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">Last Sync Result</p>
					{#if cloudSyncStore.lastResult.pushed.length > 0}
						<p class="text-sm text-green-700 dark:text-green-400">
							<Upload class="w-3 h-3 inline mr-1" />
							Pushed: {cloudSyncStore.lastResult.pushed.join(', ')}
						</p>
					{/if}
					{#if cloudSyncStore.lastResult.pulled.length > 0}
						<p class="text-sm text-blue-700 dark:text-blue-400">
							<Download class="w-3 h-3 inline mr-1" />
							Pulled: {cloudSyncStore.lastResult.pulled.join(', ')}
						</p>
					{/if}
					{#if cloudSyncStore.lastResult.conflicts.length > 0}
						<p class="text-sm text-amber-700 dark:text-amber-400 mt-1">
							<AlertCircle class="w-3 h-3 inline mr-1" />
							Conflicts: {cloudSyncStore.lastResult.conflicts.join(', ')}
						</p>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>
