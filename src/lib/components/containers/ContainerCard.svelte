<script lang="ts">
	import { Star, Pencil, Trash2, Play, Square, RotateCw, Eye, Loader2 } from 'lucide-svelte';
	import ContainerStatus from './ContainerStatus.svelte';

	type Container = {
		id: number;
		name: string;
		description?: string;
		containerType: string;
		image?: string;
		icon?: string;
		isFavorite?: boolean;
		dockerfile?: string;
		ports?: any[];
		volumes?: any[];
		env?: Record<string, string>;
		tags?: string[];
	};

	type Props = {
		container: Container;
		status?: string;
		loading?: boolean;
		onEdit: (container: Container) => void;
		onDelete: (container: Container) => void;
		onFavoriteToggle?: (container: Container) => void;
		onViewDetail?: (container: Container) => void;
		onStart?: (container: Container) => void;
		onStop?: (container: Container) => void;
		onRestart?: (container: Container) => void;
	};

	let { container, status, loading = false, onEdit, onDelete, onFavoriteToggle, onViewDetail, onStart, onStop, onRestart }: Props = $props();

	const typeLabels: Record<string, string> = {
		docker: 'Docker',
		devcontainer: 'Dev Container',
		custom: 'Custom'
	};

	const displayIcon = $derived(container.icon || '\u{1F4E6}');
	const isRunning = $derived(status === 'running');
	const isStopped = $derived(status === 'stopped' || status === 'exited' || status === 'created');
	const isNotCreated = $derived(!status || status === 'not_created' || status === 'unknown');
</script>

<div class="card group relative hover:shadow-md transition-shadow duration-200 {loading ? 'opacity-75' : ''}">
	<div class="flex items-start gap-3">
		<div class="text-2xl shrink-0">{displayIcon}</div>
		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2 flex-wrap">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">{container.name}</h3>
				<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400 shrink-0">
					{typeLabels[container.containerType] || container.containerType}
				</span>
				{#if loading}
					<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-700 dark:bg-yellow-900/50 dark:text-yellow-400">
						<Loader2 class="w-3 h-3 animate-spin" />
						Working...
					</span>
				{:else if status}
					<ContainerStatus {status} />
				{/if}
			</div>
			{#if loading}
				<p class="text-sm text-yellow-600 dark:text-yellow-400 mt-1">Pulling image and starting container...</p>
			{:else}
				{#if container.description}
					<p class="text-sm text-gray-500 dark:text-gray-400 mt-1 line-clamp-1">{container.description}</p>
				{/if}
				{#if container.image}
					<p class="text-xs text-gray-400 dark:text-gray-500 mt-1 font-mono truncate">{container.image}</p>
				{/if}
			{/if}
		</div>
		<div class="flex items-center gap-1 shrink-0">
			<!-- Lifecycle actions -->
			{#if loading}
				<div class="p-2">
					<Loader2 class="w-4 h-4 animate-spin text-primary-500" />
				</div>
			{:else if isRunning}
				<button aria-label="Stop container" onclick={(e) => { e.stopPropagation(); onStop?.(container); }} class="btn btn-ghost p-2 text-red-500 hover:text-red-700" title="Stop">
					<Square class="w-4 h-4" />
				</button>
				<button aria-label="Restart container" onclick={(e) => { e.stopPropagation(); onRestart?.(container); }} class="btn btn-ghost p-2 text-gray-500 dark:text-gray-400" title="Restart">
					<RotateCw class="w-4 h-4" />
				</button>
			{:else if isStopped || isNotCreated}
				<button aria-label="Start container" onclick={(e) => { e.stopPropagation(); onStart?.(container); }} class="btn btn-ghost p-2 text-green-600 dark:text-green-400" title="Start">
					<Play class="w-4 h-4" />
				</button>
			{/if}

			<!-- View detail -->
			<button aria-label="View container details" onclick={() => onViewDetail?.(container)} class="btn btn-ghost p-2 text-gray-500 dark:text-gray-400" title="Details" disabled={loading}>
				<Eye class="w-4 h-4" />
			</button>

			<!-- Favorite -->
			<button
				aria-label={container.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
				onclick={() => onFavoriteToggle?.(container)}
				class="p-2 rounded-md transition-colors hover:bg-yellow-50 dark:hover:bg-yellow-900/20 {container.isFavorite ? 'text-yellow-500' : 'text-gray-400 dark:text-gray-500'}"
				disabled={loading}
			>
				<Star class="w-4 h-4" fill={container.isFavorite ? 'currentColor' : 'none'} />
			</button>

			<!-- Edit / Delete -->
			<button aria-label="Edit container" onclick={() => onEdit(container)} class="btn btn-ghost p-2 text-gray-500 dark:text-gray-400" disabled={loading}>
				<Pencil class="w-4 h-4" />
			</button>
			<button aria-label="Delete container" onclick={() => onDelete(container)} class="btn btn-ghost p-2 text-red-500 hover:text-red-700" disabled={loading}>
				<Trash2 class="w-4 h-4" />
			</button>
		</div>
	</div>
</div>
