<script lang="ts">
	import { Heart, Pencil, Trash2 } from 'lucide-svelte';
	import { containerLibrary, notifications } from '$lib/stores';
	import type { Container, ContainerStatus } from '$lib/types';
	import ContainerStatusBadge from './ContainerStatus.svelte';
	import ContainerActions from './ContainerActions.svelte';

	let { container, onEdit, onDelete }: {
		container: Container;
		onEdit: (container: Container) => void;
		onDelete: (container: Container) => void;
	} = $props();

	const status = $derived(containerLibrary.getStatus(container.id));
	const dockerStatus = $derived(status?.dockerStatus || 'not_created');

	const typeLabels: Record<string, string> = {
		docker: 'Docker',
		devcontainer: 'Dev Container',
		custom: 'Custom'
	};
	let actionInProgress = $state(false);

	async function withAction(fn: () => Promise<void>) {
		if (actionInProgress) return;
		actionInProgress = true;
		try { await fn(); } finally { actionInProgress = false; }
	}

	async function handleBuild() {
		await withAction(async () => {
			await containerLibrary.buildImage(container.id);
			notifications.success(`Image built for ${container.name}`);
		});
	}

	async function handleStart() {
		await withAction(async () => {
			await containerLibrary.startContainer(container.id);
			notifications.success(`${container.name} started`);
		});
	}

	async function handleStop() {
		await withAction(async () => {
			await containerLibrary.stopContainer(container.id);
			notifications.success(`${container.name} stopped`);
		});
	}

	async function handleRestart() {
		await withAction(async () => {
			await containerLibrary.restartContainer(container.id);
			notifications.success(`${container.name} restarted`);
		});
	}

	async function handleRemove() {
		await withAction(async () => {
			await containerLibrary.removeContainer(container.id);
			notifications.success(`Docker container removed for ${container.name}`);
		});
	}

	async function handleToggleFavorite() {
		try {
			await containerLibrary.toggleFavorite(container.id);
		} catch (e) {
			notifications.error(`Failed to toggle favorite: ${e}`);
		}
	}
</script>

<div class="card group hover:shadow-md transition-all duration-200">
	<div class="flex items-start justify-between">
		<div class="flex items-center gap-3 min-w-0">
			<div class="w-10 h-10 rounded-xl bg-cyan-100 dark:bg-cyan-900/50 flex items-center justify-center text-lg shrink-0">
				{container.icon || '📦'}
			</div>
			<div class="min-w-0">
				<div class="flex items-center gap-2">
					<h3 class="font-medium text-gray-900 dark:text-white truncate">{container.name}</h3>
					<ContainerStatusBadge status={dockerStatus} />
				</div>
				{#if container.description}
					<p class="text-sm text-gray-500 dark:text-gray-400 truncate mt-0.5">{container.description}</p>
				{/if}
				<div class="flex items-center gap-2 mt-1">
					<span class="text-xs px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">
						{typeLabels[container.containerType] || container.containerType}
					</span>
					{#if container.image}
						<span class="text-xs text-gray-400 dark:text-gray-500 truncate">{container.image}</span>
					{/if}
				</div>
			</div>
		</div>
		<div class="flex items-center gap-1 shrink-0 ml-2">
			<ContainerActions
				status={dockerStatus}
				disabled={actionInProgress}
				onBuild={container.dockerfile ? handleBuild : undefined}
				onStart={handleStart}
				onStop={handleStop}
				onRestart={handleRestart}
				onRemove={handleRemove}
			/>
			<button onclick={handleToggleFavorite}
				class="p-1.5 rounded-lg transition-colors {container.isFavorite ? 'text-rose-500 hover:text-rose-600' : 'text-gray-300 hover:text-rose-400 dark:text-gray-600 dark:hover:text-rose-400'}"
				aria-label={container.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
				aria-pressed={container.isFavorite}>
				<Heart class="w-4 h-4" fill={container.isFavorite ? 'currentColor' : 'none'} aria-hidden="true" />
			</button>
			<button onclick={() => onEdit(container)} class="p-1.5 rounded-lg text-gray-400 hover:text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 transition-colors" aria-label="Edit container">
				<Pencil class="w-4 h-4" aria-hidden="true" />
			</button>
			<button onclick={() => onDelete(container)} class="p-1.5 rounded-lg text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors" aria-label="Delete container">
				<Trash2 class="w-4 h-4" aria-hidden="true" />
			</button>
		</div>
	</div>
</div>
