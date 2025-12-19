<script lang="ts">
	import type { SubAgent } from '$lib/types';
	import { subagentLibrary } from '$lib/stores';
	import SubAgentCard from './SubAgentCard.svelte';
	import { SearchBar } from '$lib/components/shared';
	import { Bot } from 'lucide-svelte';

	type Props = {
		onEdit?: (subagent: SubAgent) => void;
		onDelete?: (subagent: SubAgent) => void;
	};

	let { onEdit, onDelete }: Props = $props();
</script>

<div class="space-y-4">
	<!-- Filters -->
	<div class="flex items-center gap-4">
		<div class="flex-1 max-w-sm">
			<SearchBar
				bind:value={subagentLibrary.searchQuery}
				placeholder="Search sub-agents..."
			/>
		</div>

		<div class="text-sm text-gray-500 dark:text-gray-400">
			{subagentLibrary.subagents.length} sub-agent{subagentLibrary.subagents.length !== 1 ? 's' : ''}
		</div>
	</div>

	<!-- SubAgent Grid -->
	{#if subagentLibrary.isLoading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
		</div>
	{:else if subagentLibrary.filteredSubAgents.length === 0}
		<div class="text-center py-12">
			<Bot class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-4" />
			{#if subagentLibrary.searchQuery}
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No matching sub-agents</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					Try adjusting your search
				</p>
			{:else}
				<h3 class="text-lg font-medium text-gray-900 dark:text-white">No sub-agents in library</h3>
				<p class="text-gray-500 dark:text-gray-400 mt-1">
					Add your first custom sub-agent to get started
				</p>
			{/if}
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
			{#each subagentLibrary.filteredSubAgents as subagent (subagent.id)}
				<SubAgentCard
					{subagent}
					{onEdit}
					{onDelete}
				/>
			{/each}
		</div>
	{/if}
</div>
