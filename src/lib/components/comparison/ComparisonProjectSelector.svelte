<script lang="ts">
	import type { ProjectSummary } from '$lib/types';
	import { projectTotalTokens } from '$lib/types/session';
	import { formatCompactNumber } from '$lib/types/usage';
	import { comparisonStore, PROJECT_COLORS } from '$lib/stores/comparisonStore.svelte';

	type Props = {
		projects: ProjectSummary[];
	};

	let { projects }: Props = $props();

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-2).join('/');
	}

	const selectedCount = $derived(comparisonStore.selectedFolders.size);

	function colorForProject(folder: string): string | null {
		const folders = [...comparisonStore.selectedFolders];
		const idx = folders.indexOf(folder);
		if (idx === -1) return null;
		return PROJECT_COLORS[idx] ?? '#6b7280';
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
	<div class="flex items-center justify-between mb-3">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">
			Select Projects ({selectedCount}/5 selected)
		</h3>
		{#if selectedCount > 0}
			<button
				onclick={() => comparisonStore.clearSelection()}
				class="text-xs text-primary-500 hover:text-primary-600 font-medium"
			>
				Clear all
			</button>
		{/if}
	</div>

	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
		{#each projects as project}
			{@const isSelected = comparisonStore.selectedFolders.has(project.folderName)}
			{@const isDisabled = !isSelected && selectedCount >= 5}
			{@const color = colorForProject(project.folderName)}
			<button
				onclick={() => comparisonStore.toggleProject(project.folderName)}
				disabled={isDisabled}
				class="w-full text-left px-3 py-2.5 rounded-lg transition-colors flex items-center gap-3
					{isSelected
					? 'bg-primary-50 border border-primary-200 dark:bg-primary-900/30 dark:border-primary-700'
					: isDisabled
						? 'opacity-40 cursor-not-allowed border border-transparent'
						: 'hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent'}"
			>
				<!-- Color dot / checkbox -->
				<div
					class="w-4 h-4 rounded-full border-2 flex-shrink-0 flex items-center justify-center
						{isSelected ? 'border-transparent' : 'border-gray-300 dark:border-gray-600'}"
					style={isSelected ? `background: ${color}` : ''}
				>
					{#if isSelected}
						<svg class="w-2.5 h-2.5 text-white" viewBox="0 0 12 12" fill="none">
							<path d="M2 6l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
						</svg>
					{/if}
				</div>

				<div class="flex-1 min-w-0">
					<p class="text-sm font-medium truncate {isSelected
						? 'text-primary-700 dark:text-primary-300'
						: 'text-gray-900 dark:text-white'}">
						{shortPath(project.inferredPath)}
					</p>
					<div class="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400 mt-0.5">
						<span>{project.sessionCount} sessions</span>
						<span>{formatCompactNumber(projectTotalTokens(project))} tokens</span>
					</div>
				</div>
			</button>
		{/each}
	</div>
</div>
