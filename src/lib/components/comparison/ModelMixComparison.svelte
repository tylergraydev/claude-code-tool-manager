<script lang="ts">
	import type { ProjectComparisonData } from '$lib/stores/comparisonStore.svelte';
	import { getModelColor, formatModelName } from '$lib/types/usage';

	type Props = {
		data: ProjectComparisonData[];
		allModels: string[];
	};

	let { data, allModels }: Props = $props();

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-2).join('/');
	}

	interface ModelSummary {
		modelId: string;
		name: string;
		color: string;
		usedBy: number;
	}

	const modelSummaries = $derived.by((): ModelSummary[] => {
		return allModels.map((modelId) => {
			const usedBy = data.filter((d) => d.project.modelsUsed.includes(modelId)).length;
			return {
				modelId,
				name: formatModelName(modelId),
				color: getModelColor(modelId),
				usedBy
			};
		}).sort((a, b) => b.usedBy - a.usedBy);
	});
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">Model Mix</h3>

	{#if data.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			Select projects to compare
		</div>
	{:else}
		<div class="space-y-3">
			<!-- Per-project model pills -->
			{#each data as d}
				<div class="flex items-center gap-3">
					<div class="flex items-center gap-1.5 min-w-[140px] flex-shrink-0">
						<div class="w-2.5 h-2.5 rounded-full flex-shrink-0" style="background: {d.color}"></div>
						<span class="text-sm text-gray-700 dark:text-gray-300 truncate">
							{shortPath(d.project.inferredPath)}
						</span>
					</div>
					<div class="flex flex-wrap gap-1.5">
						{#each d.project.modelsUsed as modelId}
							<span
								class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium text-white"
								style="background: {getModelColor(modelId)}"
							>
								{formatModelName(modelId)}
							</span>
						{/each}
						{#if d.project.modelsUsed.length === 0}
							<span class="text-xs text-gray-400 dark:text-gray-500">No models recorded</span>
						{/if}
					</div>
				</div>
			{/each}

			<!-- Summary -->
			{#if modelSummaries.length > 0}
				<div class="border-t border-gray-200 dark:border-gray-700 pt-3 mt-3">
					<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2">Summary</p>
					<div class="space-y-1">
						{#each modelSummaries as ms}
							<div class="flex items-center gap-2 text-xs">
								<div class="w-2 h-2 rounded-full flex-shrink-0" style="background: {ms.color}"></div>
								<span class="text-gray-700 dark:text-gray-300 font-medium">{ms.name}</span>
								<span class="text-gray-400 dark:text-gray-500">
									used by {ms.usedBy}/{data.length} project{data.length !== 1 ? 's' : ''}
								</span>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>
