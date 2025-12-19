<script lang="ts">
	import type { Project } from '$lib/types';
	import type { Mcp } from '$lib/types';
	import { dragDrop, projectsStore, notifications } from '$lib/stores';
	import { FolderOpen, MoreVertical, Trash2, RefreshCw, ExternalLink, X } from 'lucide-svelte';

	type Props = {
		project: Project;
		onRemove?: (project: Project) => void;
	};

	let { project, onRemove }: Props = $props();

	let isOver = $state(false);
	let showMenu = $state(false);

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		if (dragDrop.isDragging) {
			isOver = true;
			dragDrop.setDropTarget({ type: 'project', projectId: project.id });
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
				// Check if already assigned
				const alreadyAssigned = project.assignedMcps.some((a) => a.mcpId === mcp.id);
				if (alreadyAssigned) {
					notifications.warning(`${mcp.name} is already assigned to this project`);
				} else {
					await projectsStore.assignMcpToProject(project.id, mcp.id);
					await projectsStore.syncProjectConfig(project.id);
					notifications.success(`Added ${mcp.name} to ${project.name}`);
				}
			} catch (err) {
				notifications.error('Failed to assign MCP');
				console.error(err);
			}
		}

		dragDrop.endDrag();
	}

	async function handleRemoveMcp(mcpId: number) {
		try {
			await projectsStore.removeMcpFromProject(project.id, mcpId);
			await projectsStore.syncProjectConfig(project.id);
			notifications.success('MCP removed from project');
		} catch (err) {
			notifications.error('Failed to remove MCP');
		}
	}

	async function handleToggleMcp(assignmentId: number, enabled: boolean) {
		try {
			await projectsStore.toggleProjectMcp(assignmentId, enabled);
			await projectsStore.syncProjectConfig(project.id);
		} catch (err) {
			notifications.error('Failed to toggle MCP');
		}
	}

	function closeMenu() {
		showMenu = false;
	}
</script>

<svelte:window onclick={closeMenu} />

<div
	class="card transition-all duration-200"
	class:ring-2={isOver}
	class:ring-primary-500={isOver}
	class:ring-offset-2={isOver}
	class:bg-primary-50={isOver}
	class:dark:bg-primary-900/20={isOver}
	ondragover={handleDragOver}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
	role="region"
	aria-label="Project drop zone"
>
	<div class="flex items-start gap-3">
		<div class="flex-shrink-0 w-10 h-10 rounded-xl bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
			<FolderOpen class="w-5 h-5 text-amber-600 dark:text-amber-400" />
		</div>

		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">
					{project.name}
				</h3>
				{#if project.hasMcpFile}
					<span class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300">
						.mcp.json
					</span>
				{/if}
			</div>
			<p class="text-xs text-gray-500 dark:text-gray-400 truncate mt-0.5 font-mono">
				{project.path}
			</p>
		</div>

		<div class="relative">
			<button
				onclick={(e) => {
					e.stopPropagation();
					showMenu = !showMenu;
				}}
				class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<MoreVertical class="w-4 h-4" />
			</button>

			{#if showMenu}
				<div
					class="absolute right-0 top-full mt-1 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 z-10"
					onclick={(e) => e.stopPropagation()}
				>
					<button
						onclick={() => {
							projectsStore.syncProjectConfig(project.id);
							notifications.success('Config synced');
							closeMenu();
						}}
						class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
					>
						<RefreshCw class="w-4 h-4" />
						Sync Config
					</button>
					<button
						onclick={() => {
							// Would open in file explorer
							closeMenu();
						}}
						class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
					>
						<ExternalLink class="w-4 h-4" />
						Open Folder
					</button>
					{#if onRemove}
						<button
							onclick={() => {
								onRemove(project);
								closeMenu();
							}}
							class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20"
						>
							<Trash2 class="w-4 h-4" />
							Remove
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</div>

	<!-- Assigned MCPs -->
	<div class="mt-4">
		{#if project.assignedMcps.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each project.assignedMcps as assignment (assignment.id)}
					<div
						class="inline-flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs font-medium
							{assignment.isEnabled
								? 'bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
								: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400'}"
					>
						<button
							onclick={() => handleToggleMcp(assignment.id, !assignment.isEnabled)}
							class="w-3 h-3 rounded-full border transition-colors
								{assignment.isEnabled
									? 'bg-primary-500 border-primary-500'
									: 'bg-transparent border-gray-400'}"
							title={assignment.isEnabled ? 'Disable' : 'Enable'}
						>
							{#if assignment.isEnabled}
								<span class="sr-only">Enabled</span>
							{/if}
						</button>
						<span class:line-through={!assignment.isEnabled}>
							{assignment.mcp.name}
						</span>
						<button
							onclick={() => handleRemoveMcp(assignment.mcpId)}
							class="p-0.5 hover:bg-primary-200 dark:hover:bg-primary-800 rounded"
							title="Remove from project"
						>
							<X class="w-3 h-3" />
						</button>
					</div>
				{/each}
			</div>
		{:else}
			<p class="text-sm text-gray-400 dark:text-gray-500 italic">
				{dragDrop.isDragging ? 'Drop MCP here to assign' : 'No MCPs assigned - drag from library'}
			</p>
		{/if}
	</div>
</div>
