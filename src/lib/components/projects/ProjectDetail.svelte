<script lang="ts">
	import type { Project, Mcp } from '$lib/types';
	import { mcpLibrary, projectsStore, notifications } from '$lib/stores';
	import { X, Plus, Minus, FolderOpen, Plug, Globe, Server } from 'lucide-svelte';

	type Props = {
		project: Project;
		onClose: () => void;
	};

	let { project: initialProject, onClose }: Props = $props();

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

	// Get current project from store (updates after loadProjects)
	let project = $derived(
		projectsStore.getProjectById(initialProject.id) ?? initialProject
	);

	// Get assigned MCP IDs for this project
	let assignedMcpIds = $derived(project.assignedMcps.map((a) => a.mcpId));

	// Available MCPs (in library but not assigned to this project)
	let availableMcps = $derived(
		mcpLibrary.mcps.filter((mcp) => !assignedMcpIds.includes(mcp.id))
	);

	async function handleAdd(mcp: Mcp) {
		try {
			await projectsStore.assignMcpToProject(project.id, mcp.id);
			await projectsStore.syncProjectConfig(project.id);
			notifications.success(`Added ${mcp.name} to ${project.name}`);
		} catch (err) {
			notifications.error('Failed to add MCP');
			console.error(err);
		}
	}

	async function handleRemove(mcpId: number) {
		try {
			const mcp = mcpLibrary.getMcpById(mcpId);
			await projectsStore.removeMcpFromProject(project.id, mcpId);
			await projectsStore.syncProjectConfig(project.id);
			notifications.success(`Removed ${mcp?.name || 'MCP'} from ${project.name}`);
		} catch (err) {
			notifications.error('Failed to remove MCP');
			console.error(err);
		}
	}

	async function handleToggle(assignmentId: number, enabled: boolean) {
		try {
			await projectsStore.toggleProjectMcp(assignmentId, enabled);
			await projectsStore.syncProjectConfig(project.id);
		} catch (err) {
			notifications.error('Failed to toggle MCP');
			console.error(err);
		}
	}
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
	<div
		class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[85vh] flex flex-col"
		onclick={(e) => e.stopPropagation()}
	>
		<!-- Header -->
		<div class="flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-3">
				<div class="w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
					<FolderOpen class="w-5 h-5 text-amber-600 dark:text-amber-400" />
				</div>
				<div>
					<h2 class="text-xl font-semibold text-gray-900 dark:text-white">{project.name}</h2>
					<p class="text-sm text-gray-500 dark:text-gray-400 font-mono">{project.path}</p>
				</div>
			</div>
			<button
				onclick={onClose}
				class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-6 space-y-6">
			<!-- Assigned MCPs -->
			<div>
				<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
					Assigned MCPs ({project.assignedMcps.length})
				</h3>
				{#if project.assignedMcps.length > 0}
					<div class="space-y-2">
						{#each project.assignedMcps as assignment (assignment.id)}
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
										title="Remove from project"
									>
										<Minus class="w-4 h-4" />
									</button>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
						<p class="text-gray-500 dark:text-gray-400">No MCPs assigned yet</p>
						<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Add MCPs from the library below</p>
					</div>
				{/if}
			</div>

			<!-- Available MCPs -->
			<div>
				<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
					Available MCPs ({availableMcps.length})
				</h3>
				{#if availableMcps.length > 0}
					<div class="space-y-2">
						{#each availableMcps as mcp (mcp.id)}
							<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
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
									onclick={() => handleAdd(mcp)}
									class="p-1.5 text-gray-400 hover:text-green-500 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition-colors"
									title="Add to project"
								>
									<Plus class="w-4 h-4" />
								</button>
							</div>
						{/each}
					</div>
				{:else}
					<div class="text-center py-6 bg-gray-50 dark:bg-gray-700/30 rounded-lg">
						<p class="text-gray-500 dark:text-gray-400">All MCPs are assigned</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>
