<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import InsightsReportViewer from '$lib/components/analytics/InsightsReportViewer.svelte';
	import SessionQualityCards from '$lib/components/analytics/SessionQualityCards.svelte';
	import FrictionTrendsChart from '$lib/components/analytics/FrictionTrendsChart.svelte';
	import SessionSummaryList from '$lib/components/analytics/SessionSummaryList.svelte';
	import { insightsStore } from '$lib/stores';
	import { FileQuestion } from 'lucide-svelte';

	onMount(() => {
		insightsStore.load();
	});

	function handleRefresh() {
		insightsStore.load();
	}
</script>

<Header title="Insights" subtitle="Session quality, friction analysis, and Claude's insights report" />

<div class="flex-1 overflow-auto p-6 space-y-6">
	<!-- Report Section -->
	{#if insightsStore.isLoadingReport}
		<div class="flex items-center justify-center py-12">
			<div
				class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
			></div>
		</div>
	{:else if insightsStore.reportError}
		<div
			class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
		>
			{insightsStore.reportError}
		</div>
	{:else if insightsStore.reportExists && insightsStore.reportHtml}
		<InsightsReportViewer
			htmlContent={insightsStore.reportHtml}
			filePath={insightsStore.reportFilePath}
			onRefresh={handleRefresh}
		/>
	{:else}
		<div
			class="text-center py-10 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
		>
			<div class="text-gray-400 dark:text-gray-500">
				<FileQuestion class="w-10 h-10 mx-auto mb-3 opacity-50" />
				<p class="text-sm font-medium">No insights report found</p>
				<p class="text-xs mt-1">
					Run <code class="bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">/insights</code> in Claude Code to generate the report.
				</p>
			</div>
		</div>
	{/if}

	<!-- Facets Section -->
	{#if insightsStore.isLoadingFacets}
		<div class="flex items-center justify-center py-12">
			<div
				class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
			></div>
		</div>
	{:else if insightsStore.facetsError}
		<div
			class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
		>
			{insightsStore.facetsError}
		</div>
	{:else if insightsStore.facetsExist && insightsStore.facets.length > 0}
		<SessionQualityCards facets={insightsStore.facets} />

		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<FrictionTrendsChart facets={insightsStore.facets} />
			<SessionSummaryList facets={insightsStore.facets} />
		</div>
	{:else if !insightsStore.isLoadingReport}
		<div
			class="text-center py-10 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
		>
			<div class="text-gray-400 dark:text-gray-500">
				<FileQuestion class="w-10 h-10 mx-auto mb-3 opacity-50" />
				<p class="text-sm font-medium">No session quality data found</p>
				<p class="text-xs mt-1">
					Session facets are stored in <code class="bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">~/.claude/usage-data/facets/</code>
				</p>
				<p class="text-xs mt-1">Use Claude Code to generate session data, then refresh this page.</p>
			</div>
		</div>
	{/if}
</div>
