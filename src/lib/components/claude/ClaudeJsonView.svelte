<script lang="ts">
	import { claudeJson, notifications, type ClaudeJsonMcp } from '$lib/stores';
	import { FileJson, FolderOpen, Plug, Globe, Server, X, RefreshCw, ChevronDown, ChevronRight } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let expandedProjects = $state<Set<string>>(new Set());

	onMount(() => {
		claudeJson.loadAll();
	});

	const typeIcons = {
		stdio: Plug,
		sse: Globe,
		http: Server
	};

	const typeColors = {
		stdio: 'bg-purple-100 text-purple-600 dark:bg-purple-900/50 dark:text-purple-400',
		sse: 'bg-green-100 text-green-600 dark:bg-green-900/50 dark:text-green-400',
		http: 'bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400'
	};

	function toggleProject(path: string) {
		const newSet = new Set(expandedProjects);
		if (newSet.has(path)) {
			newSet.delete(path);
		} else {
			newSet.add(path);
		}
		expandedProjects = newSet;
	}

	async function handleToggle(mcp: ClaudeJsonMcp) {
		if (!mcp.projectPath) return;
		try {
			await claudeJson.toggleMcp(mcp.projectPath, mcp.name, !mcp.isEnabled);
			notifications.success(`${mcp.name} ${mcp.isEnabled ? 'disabled' : 'enabled'}`);
		} catch {
			notifications.error('Failed to toggle MCP');
		}
	}

	async function handleRemove(mcp: ClaudeJsonMcp) {
		try {
			if (mcp.projectPath) {
				await claudeJson.removeMcpFromProject(mcp.projectPath, mcp.name);
			} else {
				await claudeJson.removeGlobalMcp(mcp.name);
			}
			notifications.success(`Removed ${mcp.name}`);
		} catch {
			notifications.error('Failed to remove MCP');
		}
	}

	function getProjectName(path: string): string {
		return path.split(/[/\\]/).pop() || path;
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="w-10 h-10 rounded-xl bg-orange-100 dark:bg-orange-900/50 flex items-center justify-center">
				<FileJson class="w-5 h-5 text-orange-600 dark:text-orange-400" />
			</div>
			<div>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Claude.json Config</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400">
					MCPs configured directly in ~/.claude.json
				</p>
			</div>
		</div>
		<button onclick={() => claudeJson.loadAll()} class="btn btn-secondary">
			<RefreshCw class="w-4 h-4 mr-2" />
			Refresh
		</button>
	</div>

	{#if claudeJson.isLoading}
		<div class="flex items-center justify-center py-8">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
		</div>
	{:else if claudeJson.error}
		<div class="card bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800">
			<p class="text-red-600 dark:text-red-400">{claudeJson.error}</p>
		</div>
	{:else}
		<!-- Global MCPs -->
		{#if claudeJson.globalMcps.length > 0}
			<div class="card">
				<h3 class="text-md font-medium text-gray-900 dark:text-white mb-4">Global MCPs</h3>
				<div class="space-y-2">
					{#each claudeJson.globalMcps as mcp (mcp.name)}
						<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
							<div class="flex items-center gap-3">
								<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center">
									<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
								</div>
								<div>
									<span class="font-medium text-gray-900 dark:text-white">{mcp.name}</span>
									<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({mcp.type})</span>
								</div>
							</div>
							<button
								onclick={() => handleRemove(mcp)}
								class="p-1.5 text-gray-400 hover:text-red-500 rounded"
								title="Remove"
							>
								<X class="w-4 h-4" />
							</button>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Project MCPs -->
		{#if claudeJson.projects.length > 0}
			<div class="space-y-3">
				<h3 class="text-md font-medium text-gray-900 dark:text-white">Project MCPs</h3>
				{#each claudeJson.projects as project (project.path)}
					<div class="card">
						<button
							onclick={() => toggleProject(project.path)}
							class="w-full flex items-center justify-between"
						>
							<div class="flex items-center gap-3">
								<div class="w-8 h-8 rounded-lg bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
									<FolderOpen class="w-4 h-4 text-amber-600 dark:text-amber-400" />
								</div>
								<div class="text-left">
									<span class="font-medium text-gray-900 dark:text-white">{getProjectName(project.path)}</span>
									<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{project.path}</p>
								</div>
							</div>
							<div class="flex items-center gap-2">
								<span class="text-sm text-gray-500 dark:text-gray-400">
									{project.mcps.length} MCP{project.mcps.length !== 1 ? 's' : ''}
								</span>
								{#if expandedProjects.has(project.path)}
									<ChevronDown class="w-5 h-5 text-gray-400" />
								{:else}
									<ChevronRight class="w-5 h-5 text-gray-400" />
								{/if}
							</div>
						</button>

						{#if expandedProjects.has(project.path)}
							<div class="mt-4 space-y-2 pl-11">
								{#each project.mcps as mcp (mcp.name)}
									<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
										<div class="flex items-center gap-3">
											<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center">
												<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
											</div>
											<div>
												<span class="font-medium text-gray-900 dark:text-white {!mcp.isEnabled ? 'line-through opacity-50' : ''}">
													{mcp.name}
												</span>
												<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({mcp.type})</span>
												{#if !mcp.isEnabled}
													<span class="ml-2 text-xs text-yellow-600 dark:text-yellow-400">disabled</span>
												{/if}
											</div>
										</div>
										<div class="flex items-center gap-2">
											<button
												onclick={() => handleToggle(mcp)}
												class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {mcp.isEnabled ? 'bg-primary-600' : 'bg-gray-200 dark:bg-gray-600'}"
												role="switch"
												aria-checked={mcp.isEnabled}
											>
												<span
													class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {mcp.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
												></span>
											</button>
											<button
												onclick={() => handleRemove(mcp)}
												class="p-1.5 text-gray-400 hover:text-red-500 rounded"
												title="Remove"
											>
												<X class="w-4 h-4" />
											</button>
										</div>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{:else if claudeJson.globalMcps.length === 0}
			<div class="card text-center py-8">
				<FileJson class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No MCPs in claude.json</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					MCPs configured through Claude Code will appear here
				</p>
			</div>
		{/if}
	{/if}
</div>
