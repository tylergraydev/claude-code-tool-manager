<script lang="ts">
	import type { Mcp } from '$lib/types';
	import { projectsStore, notifications, mcpLibrary } from '$lib/stores';
	import { Globe, RefreshCw, Plus, Minus, Plug, Server } from 'lucide-svelte';

	let showAddModal = $state(false);

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

	// Get MCPs already in global settings
	let globalMcpIds = $derived(projectsStore.globalMcps.map((g) => g.mcpId));

	// Available MCPs (not in global)
	let availableMcps = $derived(
		mcpLibrary.mcps.filter((mcp) => !globalMcpIds.includes(mcp.id))
	);

	async function handleSync() {
		try {
			await projectsStore.syncGlobalConfig();
			notifications.success('Global config synced');
		} catch {
			notifications.error('Failed to sync config');
		}
	}

	async function handleAdd(mcp: Mcp) {
		try {
			await projectsStore.addGlobalMcp(mcp.id);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Added ${mcp.name} to global settings`);
		} catch {
			notifications.error('Failed to add MCP');
		}
	}

	async function handleRemove(mcpId: number) {
		try {
			const mcp = mcpLibrary.getMcpById(mcpId);
			await projectsStore.removeGlobalMcp(mcpId);
			await projectsStore.syncGlobalConfig();
			notifications.success(`Removed ${mcp?.name || 'MCP'} from global settings`);
		} catch {
			notifications.error('Failed to remove MCP');
		}
	}

	async function handleToggle(assignmentId: number, enabled: boolean) {
		try {
			await projectsStore.toggleGlobalMcp(assignmentId, enabled);
			await projectsStore.syncGlobalConfig();
		} catch {
			notifications.error('Failed to toggle MCP');
		}
	}
</script>

<div class="space-y-4">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="w-10 h-10 rounded-xl bg-indigo-100 dark:bg-indigo-900/50 flex items-center justify-center">
				<Globe class="w-5 h-5 text-indigo-600 dark:text-indigo-400" />
			</div>
			<div>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Global Settings</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400">
					MCPs available in all projects
				</p>
			</div>
		</div>
		<div class="flex gap-2">
			<button onclick={() => (showAddModal = true)} class="btn btn-primary">
				<Plus class="w-4 h-4 mr-2" />
				Add MCP
			</button>
			<button onclick={handleSync} class="btn btn-secondary">
				<RefreshCw class="w-4 h-4 mr-2" />
				Sync
			</button>
		</div>
	</div>

	<!-- Global MCPs List -->
	<div class="card">
		{#if projectsStore.globalMcps.length > 0}
			<div class="space-y-2">
				{#each projectsStore.globalMcps as assignment (assignment.id)}
					{@const mcp = mcpLibrary.getMcpById(assignment.mcpId) ?? assignment.mcp}
					<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
						<div class="flex items-center gap-3">
							<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center">
								<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
							</div>
							<div>
								<span class="font-medium text-gray-900 dark:text-white {!assignment.isEnabled ? 'line-through opacity-50' : ''}">
									{mcp.name}
								</span>
								<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({mcp.type})</span>
							</div>
						</div>
						<div class="flex items-center gap-3">
							<!-- Toggle -->
							<button
								onclick={() => handleToggle(assignment.id, !assignment.isEnabled)}
								class="relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {assignment.isEnabled ? 'bg-primary-600' : 'bg-gray-300 dark:bg-gray-600'}"
								role="switch"
								aria-checked={assignment.isEnabled}
								title={assignment.isEnabled ? 'Disable' : 'Enable'}
							>
								<span
									class="pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out {assignment.isEnabled ? 'translate-x-4' : 'translate-x-0'}"
								></span>
							</button>
							<!-- Remove -->
							<button
								onclick={() => handleRemove(assignment.mcpId)}
								class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
								title="Remove"
							>
								<Minus class="w-4 h-4" />
							</button>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<div class="text-center py-8">
				<Globe class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No global MCPs</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1 mb-4">
					Add MCPs to make them available in all projects
				</p>
				<button onclick={() => (showAddModal = true)} class="btn btn-primary">
					<Plus class="w-4 h-4 mr-2" />
					Add MCP
				</button>
			</div>
		{/if}
	</div>
</div>

<!-- Add MCP Modal -->
{#if showAddModal}
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={() => (showAddModal = false)}>
		<div
			class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 max-h-[70vh] flex flex-col"
			onclick={(e) => e.stopPropagation()}
		>
			<div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Add Global MCP</h3>
				<button
					onclick={() => (showAddModal = false)}
					class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xl"
				>
					&times;
				</button>
			</div>
			<div class="flex-1 overflow-auto p-4">
				{#if availableMcps.length > 0}
					<div class="space-y-2">
						{#each availableMcps as mcp (mcp.id)}
							<button
								onclick={() => {
									handleAdd(mcp);
									showAddModal = false;
								}}
								class="w-full flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
							>
								<div class="w-8 h-8 rounded-lg {typeColors[mcp.type]} flex items-center justify-center">
									<svelte:component this={typeIcons[mcp.type]} class="w-4 h-4" />
								</div>
								<div class="flex-1">
									<span class="font-medium text-gray-900 dark:text-white">{mcp.name}</span>
									<span class="text-xs text-gray-500 dark:text-gray-400 ml-2">({mcp.type})</span>
								</div>
								<Plus class="w-4 h-4 text-gray-400" />
							</button>
						{/each}
					</div>
				{:else}
					<div class="text-center py-8 text-gray-500 dark:text-gray-400">
						All MCPs are already in global settings
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
