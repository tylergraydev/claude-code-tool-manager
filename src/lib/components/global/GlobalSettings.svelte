<script lang="ts">
	import type { Mcp } from '$lib/types';
	import { projectsStore, dragDrop, notifications, mcpLibrary } from '$lib/stores';
	import { Settings, X, RefreshCw } from 'lucide-svelte';

	let isOver = $state(false);

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (dragDrop.isDragging) {
			isOver = true;
			dragDrop.setDropTarget({ type: 'global' });
		}
	}

	function handleDragLeave() {
		isOver = false;
		dragDrop.setDropTarget(null);
	}

	async function handleDrop(e: DragEvent) {
		e.preventDefault();
		isOver = false;

		const data = e.dataTransfer?.getData('application/json');
		if (data) {
			try {
				const mcp = JSON.parse(data) as Mcp;
				const alreadyGlobal = projectsStore.globalMcps.some((g) => g.mcpId === mcp.id);
				if (alreadyGlobal) {
					notifications.warning(`${mcp.name} is already in global settings`);
				} else {
					await projectsStore.addGlobalMcp(mcp.id);
					await projectsStore.syncGlobalConfig();
					notifications.success(`Added ${mcp.name} to global settings`);
				}
			} catch (err) {
				notifications.error('Failed to add to global settings');
				console.error(err);
			}
		}

		dragDrop.endDrag();
	}

	async function handleRemove(mcpId: number) {
		try {
			await projectsStore.removeGlobalMcp(mcpId);
			await projectsStore.syncGlobalConfig();
			notifications.success('Removed from global settings');
		} catch (err) {
			notifications.error('Failed to remove from global settings');
		}
	}

	async function handleToggle(id: number, enabled: boolean) {
		try {
			await projectsStore.toggleGlobalMcp(id, enabled);
			await projectsStore.syncGlobalConfig();
		} catch (err) {
			notifications.error('Failed to toggle MCP');
		}
	}

	async function handleSync() {
		try {
			await projectsStore.syncGlobalConfig();
			notifications.success('Global config synced');
		} catch (err) {
			notifications.error('Failed to sync config');
		}
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Global MCP Settings</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400">
				MCPs enabled globally are available in all Claude Code sessions
			</p>
		</div>
		<button onclick={handleSync} class="btn btn-secondary">
			<RefreshCw class="w-4 h-4 mr-2" />
			Sync Config
		</button>
	</div>

	<!-- Drop Zone -->
	<div
		class="card transition-all duration-200 min-h-[200px]"
		class:ring-2={isOver}
		class:ring-primary-500={isOver}
		class:ring-offset-2={isOver}
		class:bg-primary-50={isOver}
		class:dark:bg-primary-900/20={isOver}
		ondragover={handleDragOver}
		ondragleave={handleDragLeave}
		ondrop={handleDrop}
		role="region"
		aria-label="Global settings drop zone"
	>
		<div class="flex items-center gap-3 mb-4">
			<div class="w-10 h-10 rounded-xl bg-indigo-100 dark:bg-indigo-900/50 flex items-center justify-center">
				<Settings class="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
			</div>
			<div>
				<h4 class="font-medium text-gray-900 dark:text-white">Global MCPs</h4>
				<p class="text-xs text-gray-500 dark:text-gray-400">~/.claude/settings.json</p>
			</div>
		</div>

		{#if projectsStore.globalMcps.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each projectsStore.globalMcps as globalMcp (globalMcp.id)}
					{@const mcp = mcpLibrary.getMcpById(globalMcp.mcpId) ?? globalMcp.mcp}
					<div
						class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium
							{globalMcp.isEnabled
								? 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300'
								: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400'}"
					>
						<button
							onclick={() => handleToggle(globalMcp.id, !globalMcp.isEnabled)}
							class="w-3.5 h-3.5 rounded-full border-2 transition-colors
								{globalMcp.isEnabled
									? 'bg-indigo-500 border-indigo-500'
									: 'bg-transparent border-gray-400'}"
							title={globalMcp.isEnabled ? 'Disable' : 'Enable'}
						/>
						<span class:line-through={!globalMcp.isEnabled} class:opacity-50={!globalMcp.isEnabled}>
							{mcp.name}
						</span>
						<span class="text-xs opacity-60">({mcp.type})</span>
						<button
							onclick={() => handleRemove(globalMcp.mcpId)}
							class="p-0.5 hover:bg-indigo-200 dark:hover:bg-indigo-800 rounded"
							title="Remove from global"
						>
							<X class="w-3.5 h-3.5" />
						</button>
					</div>
				{/each}
			</div>
		{:else}
			<div class="text-center py-8">
				<Settings class="w-10 h-10 mx-auto text-gray-300 dark:text-gray-600 mb-3" />
				<p class="text-gray-500 dark:text-gray-400">
					{dragDrop.isDragging ? 'Drop here to add to global settings' : 'No global MCPs configured'}
				</p>
				<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
					Drag MCPs from the library to add them globally
				</p>
			</div>
		{/if}
	</div>
</div>
