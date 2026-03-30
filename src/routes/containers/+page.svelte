<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { ContainerList, ContainerForm, ContainerDetail } from '$lib/components/containers';
	import { ConfirmDialog } from '$lib/components/shared';
	import { containerLibrary, notifications } from '$lib/stores';
	import type { Container, CreateContainerRequest } from '$lib/types';
	import { Plus, LayoutTemplate, Server, Search } from 'lucide-svelte';
	import { onMount, onDestroy } from 'svelte';

	let showAddContainer = $state(false);
	let editingContainer = $state<Container | null>(null);
	let deletingContainer = $state<Container | null>(null);
	let viewingContainer = $state<Container | null>(null);

	onMount(async () => {
		await containerLibrary.checkDocker();
		await containerLibrary.refreshAllStatuses();
		containerLibrary.startStatusPolling();
	});

	onDestroy(() => {
		containerLibrary.stopStatusPolling();
	});

	async function handleCreate(values: CreateContainerRequest) {
		try {
			await containerLibrary.create(values);
			showAddContainer = false;
			notifications.success('Container created successfully');
		} catch (err) {
			notifications.error(`Failed to create container: ${err}`);
		}
	}

	async function handleUpdate(values: CreateContainerRequest) {
		if (!editingContainer) return;
		try {
			await containerLibrary.update(editingContainer.id, values);
			editingContainer = null;
			notifications.success('Container updated successfully');
		} catch (err) {
			notifications.error(`Failed to update container: ${err}`);
		}
	}

	async function handleDelete() {
		if (!deletingContainer) return;
		try {
			await containerLibrary.delete(deletingContainer.id);
			notifications.success('Container deleted');
		} catch (err) {
			notifications.error(`Failed to delete container: ${err}`);
		} finally {
			deletingContainer = null;
		}
	}
</script>

<Header
	title="Dev Containers"
	subtitle="Manage containerized development environments"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex items-center justify-between mb-6">
		<div class="flex items-center gap-3">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" aria-hidden="true" />
				<input
					type="text"
					bind:value={containerLibrary.searchQuery}
					placeholder="Search containers..."
					class="input pl-9 w-64"
				/>
			</div>
			<select bind:value={containerLibrary.selectedType}
				class="input w-auto">
				<option value="all">All Types ({containerLibrary.containerCount.total})</option>
				<option value="docker">Docker ({containerLibrary.containerCount.docker})</option>
				<option value="devcontainer">Dev Container ({containerLibrary.containerCount.devcontainer})</option>
				<option value="custom">Custom ({containerLibrary.containerCount.custom})</option>
			</select>
			{#if containerLibrary.dockerAvailable === false}
				<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-400">Docker not available</span>
			{:else if containerLibrary.dockerAvailable === true}
				<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400">Docker connected</span>
			{/if}
		</div>
		<div class="flex gap-2">
			<a href="/containers/templates"
				class="btn btn-secondary gap-2">
				<LayoutTemplate class="w-4 h-4" aria-hidden="true" />
				Templates
			</a>
			<a href="/containers/hosts"
				class="btn btn-secondary gap-2">
				<Server class="w-4 h-4" aria-hidden="true" />
				Hosts
			</a>
			<button onclick={() => showAddContainer = true} class="btn btn-primary gap-2">
				<Plus class="w-4 h-4" aria-hidden="true" />
				New Container
			</button>
		</div>
	</div>

	<ContainerList
		onEdit={(container) => editingContainer = container}
		onDelete={(container) => deletingContainer = container}
	/>
</div>

<!-- Add Container Modal -->
{#if showAddContainer}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true"
		onkeydown={(e) => e.key === 'Escape' && (showAddContainer = false)}
		onclick={(e) => e.target === e.currentTarget && (showAddContainer = false)}>
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">New Container</h2>
				<ContainerForm onSubmit={handleCreate} onCancel={() => showAddContainer = false} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit Container Modal -->
{#if editingContainer}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true"
		onkeydown={(e) => e.key === 'Escape' && (editingContainer = null)}
		onclick={(e) => e.target === e.currentTarget && (editingContainer = null)}>
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-auto">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Edit Container</h2>
				<ContainerForm container={editingContainer} onSubmit={handleUpdate} onCancel={() => editingContainer = null} />
			</div>
		</div>
	</div>
{/if}

<!-- Detail Modal -->
{#if viewingContainer}
	<ContainerDetail container={viewingContainer} onClose={() => viewingContainer = null} />
{/if}

<!-- Delete Confirmation -->
<ConfirmDialog
	open={!!deletingContainer}
	title="Delete Container"
	message="Are you sure you want to delete '{deletingContainer?.name}'? This will not remove the Docker container if one exists."
	onConfirm={handleDelete}
	onCancel={() => deletingContainer = null}
/>
