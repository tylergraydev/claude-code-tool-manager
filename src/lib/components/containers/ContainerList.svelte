<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import ContainerCard from './ContainerCard.svelte';

	type Props = {
		onEdit: (container: any) => void;
		onDelete: (container: any) => void;
	};

	let { onEdit, onDelete }: Props = $props();
</script>

{#if containerLibrary.isLoading}
	<div class="flex items-center justify-center py-8">
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
			<ContainerCard {container} {onEdit} {onDelete} />
		{/each}
	</div>
{/if}
