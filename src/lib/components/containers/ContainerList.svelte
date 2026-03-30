<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import type { Container } from '$lib/types';
	import ContainerCard from './ContainerCard.svelte';
	import { Package } from 'lucide-svelte';

	let { onEdit, onDelete }: {
		onEdit: (container: Container) => void;
		onDelete: (container: Container) => void;
	} = $props();
</script>

{#if containerLibrary.isLoading}
	<div class="flex items-center justify-center py-12">
		<div class="w-6 h-6 border-2 border-primary-500 border-t-transparent rounded-full animate-spin"></div>
	</div>
{:else if containerLibrary.filteredContainers.length === 0}
	<div class="text-center py-12">
		<Package class="w-12 h-12 text-gray-300 dark:text-gray-600 mx-auto mb-3" aria-hidden="true" />
		<h3 class="text-lg font-medium text-gray-500 dark:text-gray-400">
			{containerLibrary.searchQuery ? 'No matching containers' : 'No containers yet'}
		</h3>
		<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
			{containerLibrary.searchQuery ? 'Try a different search term' : 'Create a container or use a template to get started'}
		</p>
	</div>
{:else}
	<div class="space-y-3">
		{#each containerLibrary.filteredContainers as container (container.id)}
			<ContainerCard {container} {onEdit} {onDelete} />
		{/each}
	</div>
{/if}
