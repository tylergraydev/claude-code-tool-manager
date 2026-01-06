<script lang="ts">
	import type { Command } from '$lib/types';
	import { commandLibrary } from '$lib/stores';
	import CommandCard from './CommandCard.svelte';
	import { SearchBar } from '$lib/components/shared';
	import { Terminal } from 'lucide-svelte';

	type Props = {
		onEdit?: (command: Command) => void;
		onDelete?: (command: Command) => void;
	};

	let { onEdit, onDelete }: Props = $props();
</script>

<div class="space-y-4">
	<!-- Filters -->
	<div class="flex items-center gap-4">
		<div class="flex-1 max-w-sm">
			<SearchBar
				bind:value={commandLibrary.searchQuery}
				placeholder="Search commands..."
			/>
		</div>

		<div class="text-sm text-gray-500 dark:text-gray-400">
			{commandLibrary.commands.length} command{commandLibrary.commands.length !== 1 ? 's' : ''}
		</div>
	</div>

	<!-- Command Grid -->
	{#if commandLibrary.isLoading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
		</div>
	{:else if commandLibrary.filteredCommands.length === 0}
		<div class="text-center py-12">
			<Terminal class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
			{#if commandLibrary.searchQuery}
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No matching commands</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					Try adjusting your search
				</p>
			{:else}
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No commands in library</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					Add your first slash command to get started
				</p>
			{/if}
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
			{#each commandLibrary.filteredCommands as command (command.id)}
				<CommandCard
					{command}
					{onEdit}
					{onDelete}
				/>
			{/each}
		</div>
	{/if}
</div>
