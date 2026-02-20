<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { sessionStore } from '$lib/stores';
	import { comparisonStore } from '$lib/stores/comparisonStore.svelte';
	import ComparisonProjectSelector from '$lib/components/comparison/ComparisonProjectSelector.svelte';
	import ComparisonOverviewTable from '$lib/components/comparison/ComparisonOverviewTable.svelte';
	import TokenComparisonChart from '$lib/components/comparison/TokenComparisonChart.svelte';
	import CostComparisonChart from '$lib/components/comparison/CostComparisonChart.svelte';
	import ToolUsageComparisonChart from '$lib/components/comparison/ToolUsageComparisonChart.svelte';
	import ModelMixComparison from '$lib/components/comparison/ModelMixComparison.svelte';
	import { GitCompareArrows, FileQuestion } from 'lucide-svelte';

	onMount(() => {
		if (sessionStore.projects.length === 0) {
			sessionStore.loadProjects();
		}
	});

	const hasEnoughSelected = $derived(comparisonStore.selectedFolders.size >= 2);
</script>

<Header title="Cross-Project Comparison" subtitle="Compare usage metrics across projects side-by-side" />

<div class="flex-1 overflow-auto p-6 space-y-6">
	{#if sessionStore.isLoadingProjects}
		<div class="flex items-center justify-center py-20">
			<div
				class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
			></div>
		</div>
	{:else if sessionStore.projectsError}
		<div
			class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
		>
			{sessionStore.projectsError}
		</div>
	{:else if sessionStore.projects.length === 0}
		<div
			class="text-center py-16 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
		>
			<div class="text-gray-400 dark:text-gray-500">
				<FileQuestion class="w-12 h-12 mx-auto mb-3 opacity-50" />
				<p class="text-lg font-medium">No projects found</p>
				<p class="text-sm mt-1">Use Claude Code to generate session data, then refresh this page.</p>
			</div>
		</div>
	{:else}
		<!-- Project Selector -->
		<ComparisonProjectSelector projects={sessionStore.projects} />

		{#if !hasEnoughSelected}
			<div
				class="text-center py-12 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
			>
				<div class="text-gray-400 dark:text-gray-500">
					<GitCompareArrows class="w-10 h-10 mx-auto mb-3 opacity-50" />
					<p class="text-sm font-medium">Select at least 2 projects to compare</p>
					<p class="text-xs mt-1">
						Choose 2–5 projects above to see side-by-side metrics
					</p>
				</div>
			</div>
		{:else}
			<!-- Overview Table -->
			<ComparisonOverviewTable data={comparisonStore.comparisonData} />

			<!-- Token + Cost Charts -->
			<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
				<TokenComparisonChart data={comparisonStore.comparisonData} />
				<CostComparisonChart data={comparisonStore.comparisonData} />
			</div>

			<!-- Tool Usage -->
			<ToolUsageComparisonChart
				data={comparisonStore.comparisonData}
				tools={comparisonStore.allTools}
			/>

			<!-- Model Mix -->
			<ModelMixComparison
				data={comparisonStore.comparisonData}
				allModels={comparisonStore.allModels}
			/>
		{/if}
	{/if}
</div>
