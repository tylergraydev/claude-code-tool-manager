<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import ContainerCard from './ContainerCard.svelte';

	type Props = {
		onEdit: (container: any) => void;
		onDelete: (container: any) => void;
		onViewDetail?: (container: any) => void;
	};

	let { onEdit, onDelete, onViewDetail }: Props = $props();

	function handleFavoriteToggle(container: any) {
		containerLibrary.toggleFavorite(container.id);
	}

	let actionError = $state<string | null>(null);
	let loadingIds = $state<Set<number>>(new Set());

	function setLoading(id: number, loading: boolean) {
		const next = new Set(loadingIds);
		if (loading) next.add(id); else next.delete(id);
		loadingIds = next;
	}

	async function handleStart(container: any) {
		actionError = null;
		setLoading(container.id, true);
		try {
			await containerLibrary.startContainer(container.id);
		} catch (err) {
			actionError = `Failed to start "${container.name}": ${err instanceof Error ? err.message : String(err)}`;
		} finally {
			setLoading(container.id, false);
		}
	}

	async function handleStop(container: any) {
		actionError = null;
		setLoading(container.id, true);
		try {
			await containerLibrary.stopContainer(container.id);
		} catch (err) {
			actionError = `Failed to stop "${container.name}": ${err instanceof Error ? err.message : String(err)}`;
		} finally {
			setLoading(container.id, false);
		}
	}

	async function handleRestart(container: any) {
		actionError = null;
		setLoading(container.id, true);
		try {
			await containerLibrary.restartContainer(container.id);
		} catch (err) {
			actionError = `Failed to restart "${container.name}": ${err instanceof Error ? err.message : String(err)}`;
		} finally {
			setLoading(container.id, false);
		}
	}

	function getContainerStatus(id: number): string | undefined {
		const s = containerLibrary.getStatus(id) as any;
		return s?.dockerStatus || s?.docker_status;
	}
</script>

{#if actionError}
	<div class="mb-3 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm flex items-center justify-between" role="alert">
		<span>{actionError}</span>
		<button onclick={() => actionError = null} class="btn btn-ghost text-sm px-2 py-1">Dismiss</button>
	</div>
{/if}

{#if containerLibrary.isLoading}
	<div class="flex items-center justify-center py-8" role="status" aria-label="Loading containers">
		<div class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"></div>
	</div>
{:else if containerLibrary.filteredContainers.length === 0}
	{#if containerLibrary.searchQuery}
		<div class="text-center py-8">
			<p class="text-gray-500 dark:text-gray-400 font-medium">No matching containers</p>
			<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Try a different search term</p>
		</div>
	{:else}
		<div class="text-center py-8">
			<p class="text-gray-500 dark:text-gray-400 font-medium">No containers yet</p>
			<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Create a container or use a template to get started</p>
		</div>
	{/if}
{:else}
	<div class="space-y-3">
		{#each containerLibrary.filteredContainers as container (container.id)}
			<ContainerCard
				{container}
				status={getContainerStatus(container.id)}
				loading={loadingIds.has(container.id)}
				{onEdit}
				{onDelete}
				{onViewDetail}
				onFavoriteToggle={handleFavoriteToggle}
				onStart={handleStart}
				onStop={handleStop}
				onRestart={handleRestart}
			/>
		{/each}
	</div>
{/if}
