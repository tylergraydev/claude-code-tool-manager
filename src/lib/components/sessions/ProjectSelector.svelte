<script lang="ts">
	import type { ProjectSummary } from '$lib/types';
	import { projectTotalTokens } from '$lib/types/session';
	import { formatCompactNumber } from '$lib/types/usage';
	import { FolderOpen, ChevronRight } from 'lucide-svelte';

	type Props = {
		projects: ProjectSummary[];
		selectedFolder: string | null;
		onSelect: (folder: string) => void;
	};

	let { projects, selectedFolder, onSelect }: Props = $props();

	function formatDate(iso: string | null): string {
		if (!iso) return 'N/A';
		try {
			return new Date(iso).toLocaleDateString(undefined, {
				month: 'short',
				day: 'numeric',
				year: 'numeric'
			});
		} catch {
			return iso;
		}
	}

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-2).join('/');
	}
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 flex flex-col h-full"
>
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">Projects</h3>
	<div class="space-y-1.5 flex-1 overflow-y-auto min-h-0">
		{#each projects as project}
			{@const isSelected = selectedFolder === project.folderName}
			<button
				onclick={() => onSelect(project.folderName)}
				class="w-full text-left px-3 py-2.5 rounded-lg transition-colors flex items-center gap-3
					{isSelected
					? 'bg-primary-50 border border-primary-200 dark:bg-primary-900/30 dark:border-primary-700'
					: 'hover:bg-gray-50 dark:hover:bg-gray-700/50 border border-transparent'}"
			>
				<div
					class="p-1.5 rounded-lg {isSelected
						? 'bg-primary-100 dark:bg-primary-800/50'
						: 'bg-gray-100 dark:bg-gray-700'}"
				>
					<FolderOpen
						class="w-4 h-4 {isSelected
							? 'text-primary-600 dark:text-primary-400'
							: 'text-gray-500 dark:text-gray-400'}"
					/>
				</div>
				<div class="flex-1 min-w-0">
					<p
						class="text-sm font-medium truncate {isSelected
							? 'text-primary-700 dark:text-primary-300'
							: 'text-gray-900 dark:text-white'}"
					>
						{shortPath(project.inferredPath)}
					</p>
					<div class="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400 mt-0.5">
						<span>{project.sessionCount} sessions</span>
						<span>{formatCompactNumber(projectTotalTokens(project))} tokens</span>
						{#if project.latestSession}
							<span>{formatDate(project.latestSession)}</span>
						{/if}
					</div>
				</div>
				<ChevronRight
					class="w-4 h-4 flex-shrink-0 {isSelected
						? 'text-primary-500'
						: 'text-gray-300 dark:text-gray-600'}"
				/>
			</button>
		{/each}
	</div>
</div>
