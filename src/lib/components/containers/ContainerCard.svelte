<script lang="ts">
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
		onEdit: (container: Container) => void;
		onDelete: (container: Container) => void;
		onFavoriteToggle?: (container: Container, isFavorite: boolean) => void;
	};

	let { container, onEdit, onDelete, onFavoriteToggle }: Props = $props();

	const typeLabels: Record<string, string> = {
		docker: 'Docker',
		devcontainer: 'Dev Container',
		custom: 'Custom'
	};

	const displayIcon = $derived(container.icon || '\u{1F4E6}');
</script>

<div class="card group relative hover:shadow-md transition-shadow duration-200">
	<div class="flex items-start gap-3">
		<div class="text-2xl">{displayIcon}</div>
		<div class="flex-1 min-w-0">
			<div class="flex items-center gap-2">
				<h3 class="font-medium text-gray-900 dark:text-white truncate">{container.name}</h3>
				<span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-600 dark:bg-blue-900/50 dark:text-blue-400">
					{typeLabels[container.containerType] || container.containerType}
				</span>
			</div>
			{#if container.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{container.description}</p>
			{/if}
			{#if container.image}
				<p class="text-xs text-gray-400 dark:text-gray-500 mt-1 font-mono">{container.image}</p>
			{/if}
		</div>
		<div class="flex items-center gap-1">
			<button
				aria-label={container.isFavorite ? 'Remove from favorites' : 'Add to favorites'}
				onclick={() => onFavoriteToggle?.(container, !container.isFavorite)}
				class="p-1 hover:text-yellow-500 transition-colors"
				class:text-yellow-500={container.isFavorite}
			>
				{container.isFavorite ? '\u2605' : '\u2606'}
			</button>
			<button aria-label="Edit container" onclick={() => onEdit(container)} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded">
				Edit
			</button>
			<button aria-label="Delete container" onclick={() => onDelete(container)} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-red-500">
				Delete
			</button>
		</div>
	</div>
</div>
