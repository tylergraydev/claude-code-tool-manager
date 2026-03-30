<script lang="ts">
	import { containerLibrary, notifications } from '$lib/stores';
	import type { ProjectContainer, Container } from '$lib/types';
	import ContainerStatusBadge from './ContainerStatus.svelte';
	import { Plus, X, Heart } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let { projectId }: { projectId: number } = $props();

	let projectContainers = $state<ProjectContainer[]>([]);
	let showAssignDialog = $state(false);

	async function loadProjectContainers() {
		try {
			projectContainers = await containerLibrary.getProjectContainers(projectId);
		} catch {
			// Silently handle — containers may not be loaded yet
		}
	}

	async function handleAssign(container: Container) {
		try {
			await containerLibrary.assignToProject(projectId, container.id);
			await loadProjectContainers();
			showAssignDialog = false;
			notifications.success(`${container.name} assigned to project`);
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleRemove(containerId: number) {
		try {
			await containerLibrary.removeFromProject(projectId, containerId);
			await loadProjectContainers();
			notifications.success('Container removed from project');
		} catch (e) {
			notifications.error(String(e));
		}
	}

	async function handleSetDefault(containerId: number) {
		try {
			await containerLibrary.setDefaultProjectContainer(projectId, containerId);
			await loadProjectContainers();
		} catch (e) {
			notifications.error(String(e));
		}
	}

	onMount(loadProjectContainers);

	const unassignedContainers = $derived(
		containerLibrary.containers.filter(c => !projectContainers.some(pc => pc.containerId === c.id))
	);
</script>

<div class="space-y-3">
	<div class="flex items-center justify-between">
		<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">Containers</h3>
		<button onclick={() => showAssignDialog = true}
			class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400 flex items-center gap-1">
			<Plus class="w-3 h-3" aria-hidden="true" /> Assign
		</button>
	</div>

	{#if projectContainers.length === 0}
		<p class="text-sm text-gray-400 py-2">No containers assigned</p>
	{:else}
		{#each projectContainers as pc (pc.id)}
			<div class="flex items-center justify-between p-2 rounded-lg bg-gray-50 dark:bg-gray-700/50">
				<div class="flex items-center gap-2">
					<span>{pc.container.icon || '📦'}</span>
					<span class="text-sm text-gray-900 dark:text-white">{pc.container.name}</span>
					{#if pc.isDefault}
						<Heart class="w-3 h-3 text-rose-500" fill="currentColor" aria-hidden="true" />
					{/if}
				</div>
				<div class="flex items-center gap-1">
					{#if !pc.isDefault}
						<button onclick={() => handleSetDefault(pc.containerId)} class="text-xs text-gray-300 hover:text-rose-400 dark:text-gray-600 dark:hover:text-rose-400" title="Set as default">
							<Heart class="w-3 h-3" aria-hidden="true" />
						</button>
					{/if}
					<button onclick={() => handleRemove(pc.containerId)} class="text-gray-400 hover:text-red-500">
						<X class="w-4 h-4" aria-hidden="true" />
					</button>
				</div>
			</div>
		{/each}
	{/if}
</div>

{#if showAssignDialog}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true"
		onkeydown={(e) => e.key === 'Escape' && (showAssignDialog = false)}
		onclick={(e) => e.target === e.currentTarget && (showAssignDialog = false)}>
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 p-6">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Assign Container</h3>
			{#if unassignedContainers.length === 0}
				<p class="text-sm text-gray-500">All containers are already assigned</p>
			{:else}
				<div class="space-y-2 max-h-64 overflow-auto">
					{#each unassignedContainers as container (container.id)}
						<button onclick={() => handleAssign(container)}
							class="w-full flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 text-left transition-colors">
							<span>{container.icon || '📦'}</span>
							<div>
								<p class="text-sm font-medium text-gray-900 dark:text-white">{container.name}</p>
								{#if container.description}
									<p class="text-xs text-gray-500">{container.description}</p>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{/if}
			<div class="flex justify-end mt-4">
				<button onclick={() => showAssignDialog = false}
					class="btn btn-secondary">
					Close
				</button>
			</div>
		</div>
	</div>
{/if}
