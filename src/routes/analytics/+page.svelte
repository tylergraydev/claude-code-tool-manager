<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import OverviewCards from '$lib/components/analytics/OverviewCards.svelte';
	import DailyActivityChart from '$lib/components/analytics/DailyActivityChart.svelte';
	import ModelUsageBreakdown from '$lib/components/analytics/ModelUsageBreakdown.svelte';
	import DailyTokenChart from '$lib/components/analytics/DailyTokenChart.svelte';
	import PeakHoursChart from '$lib/components/analytics/PeakHoursChart.svelte';
	import { usageStore } from '$lib/stores';
	import { RefreshCw, BarChart3, FileQuestion } from 'lucide-svelte';
	import type { DateRangeFilter } from '$lib/types';

	onMount(() => {
		usageStore.load();
	});

	function handleRefresh() {
		usageStore.load();
	}

	function handleDateRangeChange(range: DateRangeFilter) {
		usageStore.setDateRange(range);
	}
</script>

<Header title="Usage Analytics" subtitle="Visualize your Claude Code usage patterns">
	{#snippet children()}
		<button onclick={handleRefresh} class="btn btn-ghost" title="Refresh data">
			<RefreshCw class="w-4 h-4" />
		</button>
	{/snippet}
</Header>

<div class="flex-1 overflow-auto p-6 space-y-6">
	{#if usageStore.isLoading}
		<div class="flex items-center justify-center py-20">
			<div
				class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
			></div>
		</div>
	{:else if usageStore.error}
		<div
			class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
		>
			{usageStore.error}
		</div>
	{:else if !usageStore.exists}
		<div
			class="text-center py-16 bg-gray-50 dark:bg-gray-800/30 rounded-lg border-2 border-dashed border-gray-200 dark:border-gray-700"
		>
			<div class="text-gray-400 dark:text-gray-500 mb-4">
				<FileQuestion class="w-12 h-12 mx-auto mb-3 opacity-50" />
				<p class="text-lg font-medium">No usage data found</p>
				<p class="text-sm mt-1">
					Claude Code stores analytics in <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded">{usageStore.filePath}</code>
				</p>
				<p class="text-sm mt-1">Use Claude Code to generate usage data, then refresh this page.</p>
			</div>
		</div>
	{:else if usageStore.stats}
		<!-- Overview Cards -->
		<OverviewCards
			totalSessions={usageStore.stats.totalSessions}
			totalMessages={usageStore.stats.totalMessages}
			totalToolCalls={usageStore.totalToolCalls}
			firstSessionDate={usageStore.stats.firstSessionDate}
			longestSession={usageStore.stats.longestSession}
			lastComputedDate={usageStore.stats.lastComputedDate}
		/>

		<!-- Daily Activity Chart -->
		<DailyActivityChart
			data={usageStore.filteredDailyActivity}
			dateRange={usageStore.dateRange}
			onDateRangeChange={handleDateRangeChange}
		/>

		<!-- Model Usage + Peak Hours (2-column on large screens) -->
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<ModelUsageBreakdown modelUsage={usageStore.stats.modelUsage} />
			<PeakHoursChart hourCounts={usageStore.hourCountsArray} />
		</div>

		<!-- Daily Token Chart -->
		<DailyTokenChart
			data={usageStore.filteredDailyTokens}
			models={usageStore.allModels}
		/>
	{/if}
</div>
