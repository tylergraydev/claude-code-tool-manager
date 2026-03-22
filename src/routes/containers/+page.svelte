<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { ContainerList, ContainerForm, ContainerDetail, DockerHostList, NewContainerWizard } from '$lib/components/containers';
	import { ConfirmDialog } from '$lib/components/shared';
	import { containerLibrary } from '$lib/stores';
	import type { Container, CreateContainerRequest } from '$lib/types';
	import { Plus, Search, Server, AlertTriangle } from 'lucide-svelte';

	let showAddContainer = $state(false);
	let editingContainer = $state<Container | null>(null);
	let deletingContainer = $state<Container | null>(null);
	let viewingContainer = $state<Container | null>(null);
	let viewingInitialTab = $state('overview');
	let showDockerHosts = $state(false);
	let formError = $state<string | null>(null);

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (showAddContainer) { showAddContainer = false; formError = null; }
			else if (editingContainer) { editingContainer = null; formError = null; }
			else if (viewingContainer) { viewingContainer = null; }
		}
	}

	function getViewingContainerStatus(): string | undefined {
		if (!viewingContainer) return undefined;
		const s = containerLibrary.getStatus(viewingContainer.id) as any;
		return s?.dockerStatus || s?.docker_status;
	}

	onMount(async () => {
		await Promise.all([
			containerLibrary.load(),
			containerLibrary.checkDocker(),
			containerLibrary.loadDockerHosts(),
			containerLibrary.loadTemplates(),
		]);
		containerLibrary.startStatusPolling();

		// Wire up callback so toast "View Logs" opens the detail modal on the logs tab
		containerLibrary.onContainerStopped = (containerId: number) => {
			const container = containerLibrary.containers.find(c => c.id === containerId);
			if (container) {
				viewingInitialTab = 'logs';
				viewingContainer = container;
			}
		};
	});

	onDestroy(() => {
		containerLibrary.stopStatusPolling();
		containerLibrary.onContainerStopped = null;
	});

	async function handleCreateContainer(values: CreateContainerRequest) {
		formError = null;
		try {
			await containerLibrary.create(values);
			showAddContainer = false;
		} catch (err) {
			formError = err instanceof Error ? err.message : String(err);
		}
	}

	async function handleUpdateContainer(values: CreateContainerRequest) {
		if (!editingContainer) return;
		formError = null;
		try {
			await containerLibrary.update(editingContainer.id, values);
			editingContainer = null;
		} catch (err) {
			formError = err instanceof Error ? err.message : String(err);
		}
	}

	async function handleDeleteContainer() {
		if (!deletingContainer) return;
		try {
			await containerLibrary.delete(deletingContainer.id);
		} catch (err) {
			console.error('Failed to delete container:', err);
		} finally {
			deletingContainer = null;
		}
	}

	async function handleToggleFavorite(container: Container) {
		await containerLibrary.toggleFavorite(container.id);
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<Header title="Containers" subtitle="Manage Docker containers for your projects" />

<div class="flex-1 overflow-auto p-6 space-y-6">
	<!-- Docker status banner -->
	{#if !containerLibrary.dockerAvailable}
		<div class="flex items-center gap-3 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg text-yellow-700 dark:text-yellow-400 text-sm">
			<AlertTriangle class="w-4 h-4 shrink-0" />
			<span>Docker is not available. Container lifecycle operations will not work until Docker is running.</span>
		</div>
	{/if}

	<!-- Action bar -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
		<div class="flex items-center gap-3 flex-1 min-w-0">
			<div class="relative flex-1 max-w-md">
				<Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
				<input
					type="text"
					placeholder="Search containers..."
					bind:value={containerLibrary.searchQuery}
					aria-label="Search containers"
					class="input w-full pl-9 text-sm"
				/>
			</div>
			<select
				bind:value={containerLibrary.selectedType}
				aria-label="Filter by container type"
				class="input text-sm w-auto"
			>
				<option value="all">All Types</option>
				<option value="docker">Docker</option>
				<option value="devcontainer">Dev Container</option>
				<option value="custom">Custom</option>
			</select>
		</div>
		<div class="flex items-center gap-2 shrink-0">
			<button onclick={() => (showDockerHosts = !showDockerHosts)} class="btn btn-secondary text-sm">
				<Server class="w-4 h-4 mr-1" />
				Hosts
			</button>
			<button onclick={() => (showAddContainer = true)} class="btn btn-primary text-sm">
				<Plus class="w-4 h-4 mr-1" />
				New Container
			</button>
		</div>
	</div>

	<!-- Docker Hosts panel -->
	{#if showDockerHosts}
		<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
			<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">Docker Hosts</h3>
			<DockerHostList />
		</div>
	{/if}

	<!-- Container count summary -->
	{#if containerLibrary.containerCount.total > 0}
		<div class="flex items-center gap-4 text-xs text-gray-500 dark:text-gray-400">
			<span>{containerLibrary.containerCount.total} container{containerLibrary.containerCount.total !== 1 ? 's' : ''}</span>
			{#if containerLibrary.containerCount.docker > 0}<span>{containerLibrary.containerCount.docker} Docker</span>{/if}
			{#if containerLibrary.containerCount.devcontainer > 0}<span>{containerLibrary.containerCount.devcontainer} Dev Container</span>{/if}
			{#if containerLibrary.containerCount.custom > 0}<span>{containerLibrary.containerCount.custom} Custom</span>{/if}
		</div>
	{/if}

	<!-- Container list -->
	<ContainerList
		onEdit={(container) => (editingContainer = container)}
		onDelete={(container) => (deletingContainer = container)}
		onViewDetail={(container) => { viewingInitialTab = 'overview'; viewingContainer = container; }}
	/>
</div>

<!-- Add Container Modal -->
{#if showAddContainer}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={(e) => { if (e.target === e.currentTarget) { showAddContainer = false; formError = null; } }}>
		<div role="dialog" aria-modal="true" aria-labelledby="add-container-title" class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col">
			<div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
				<h2 id="add-container-title" class="text-lg font-semibold text-gray-900 dark:text-white">New Container</h2>
				<button onclick={() => { showAddContainer = false; formError = null; }} class="btn btn-ghost p-1" aria-label="Close">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
				</button>
			</div>
			<div class="p-6 overflow-auto">
				{#if formError}
					<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm" role="alert">
						{formError}
					</div>
				{/if}
				<NewContainerWizard onSubmit={handleCreateContainer} onCancel={() => { showAddContainer = false; formError = null; }} />
			</div>
		</div>
	</div>
{/if}

<!-- Edit Container Modal -->
{#if editingContainer}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={(e) => { if (e.target === e.currentTarget) { editingContainer = null; formError = null; } }}>
		<div role="dialog" aria-modal="true" aria-labelledby="edit-container-title" class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-2xl w-full mx-4 max-h-[90vh] flex flex-col">
			<div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
				<h2 id="edit-container-title" class="text-lg font-semibold text-gray-900 dark:text-white">Edit Container</h2>
				<button onclick={() => { editingContainer = null; formError = null; }} class="btn btn-ghost p-1" aria-label="Close">
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
				</button>
			</div>
			<div class="p-6 overflow-auto">
				{#if formError}
					<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm" role="alert">
						{formError}
					</div>
				{/if}
				<ContainerForm
					container={editingContainer}
					onSubmit={handleUpdateContainer}
					onCancel={() => { editingContainer = null; formError = null; }}
				/>
			</div>
		</div>
	</div>
{/if}

<!-- Container Detail Modal -->
{#if viewingContainer}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={(e) => { if (e.target === e.currentTarget) viewingContainer = null; }}>
		<div role="dialog" aria-modal="true" aria-labelledby="detail-container-title" class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col">
			<div class="p-6 overflow-auto">
				<ContainerDetail
					container={viewingContainer}
					status={getViewingContainerStatus()}
					initialTab={viewingInitialTab}
					onClose={() => (viewingContainer = null)}
				/>
			</div>
		</div>
	</div>
{/if}

<ConfirmDialog
	open={!!deletingContainer}
	title="Delete Container"
	message="Are you sure you want to delete '{deletingContainer?.name ?? ''}'? This cannot be undone."
	confirmText="Delete"
	onConfirm={handleDeleteContainer}
	onCancel={() => (deletingContainer = null)}
/>
