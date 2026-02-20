<script lang="ts">
	import type { ProjectComparisonData } from '$lib/stores/comparisonStore.svelte';
	import { formatCompactNumber, formatCost, formatModelName } from '$lib/types/usage';

	type Props = {
		data: ProjectComparisonData[];
	};

	let { data }: Props = $props();

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-2).join('/');
	}

	interface MetricRow {
		label: string;
		values: string[];
		raw: number[];
		highlightMax: boolean;
	}

	const rows = $derived.by((): MetricRow[] => {
		if (data.length === 0) return [];

		const sessions: MetricRow = {
			label: 'Sessions',
			values: data.map((d) => d.project.sessionCount.toLocaleString()),
			raw: data.map((d) => d.project.sessionCount),
			highlightMax: true
		};

		const totalTokens: MetricRow = {
			label: 'Total Tokens',
			values: data.map((d) => formatCompactNumber(d.totalTokens)),
			raw: data.map((d) => d.totalTokens),
			highlightMax: true
		};

		const inputTokens: MetricRow = {
			label: 'Input Tokens',
			values: data.map((d) => formatCompactNumber(d.project.totalInputTokens)),
			raw: data.map((d) => d.project.totalInputTokens),
			highlightMax: true
		};

		const outputTokens: MetricRow = {
			label: 'Output Tokens',
			values: data.map((d) => formatCompactNumber(d.project.totalOutputTokens)),
			raw: data.map((d) => d.project.totalOutputTokens),
			highlightMax: true
		};

		const cacheTokens: MetricRow = {
			label: 'Cache Tokens',
			values: data.map((d) =>
				formatCompactNumber(d.project.totalCacheReadTokens + d.project.totalCacheCreationTokens)
			),
			raw: data.map((d) => d.project.totalCacheReadTokens + d.project.totalCacheCreationTokens),
			highlightMax: true
		};

		const cost: MetricRow = {
			label: 'Est. Cost',
			values: data.map((d) => formatCost(d.estimatedCost)),
			raw: data.map((d) => d.estimatedCost),
			highlightMax: true
		};

		const models: MetricRow = {
			label: 'Models',
			values: data.map((d) => d.project.modelsUsed.map(formatModelName).join(', ') || 'N/A'),
			raw: data.map((d) => d.project.modelsUsed.length),
			highlightMax: false
		};

		const dateRange: MetricRow = {
			label: 'Date Range',
			values: data.map((d) => d.dateRange),
			raw: data.map(() => 0),
			highlightMax: false
		};

		return [sessions, totalTokens, inputTokens, outputTokens, cacheTokens, cost, models, dateRange];
	});

	function isMax(row: MetricRow, idx: number): boolean {
		if (!row.highlightMax) return false;
		const max = Math.max(...row.raw);
		return max > 0 && row.raw[idx] === max;
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">Overview</h3>

	<div class="overflow-x-auto">
		<table class="w-full text-sm">
			<thead>
				<tr class="border-b border-gray-200 dark:border-gray-700">
					<th class="text-left py-2 px-3 font-medium text-gray-500 dark:text-gray-400 min-w-[120px]">
						Metric
					</th>
					{#each data as d}
						<th class="text-right py-2 px-3 font-medium min-w-[100px]">
							<div class="flex items-center justify-end gap-1.5">
								<div class="w-2.5 h-2.5 rounded-full flex-shrink-0" style="background: {d.color}"></div>
								<span class="text-gray-700 dark:text-gray-300 truncate max-w-[120px]">
									{shortPath(d.project.inferredPath)}
								</span>
							</div>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each rows as row}
					<tr class="border-b border-gray-100 dark:border-gray-700/50">
						<td class="py-2 px-3 text-gray-600 dark:text-gray-400 font-medium">
							{row.label}
						</td>
						{#each row.values as value, i}
							<td
								class="text-right py-2 px-3
									{isMax(row, i)
									? 'font-bold text-primary-600 dark:text-primary-400'
									: 'text-gray-700 dark:text-gray-300'}"
							>
								{value}
							</td>
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
