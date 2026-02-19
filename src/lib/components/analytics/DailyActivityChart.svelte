<script lang="ts">
	import type { DailyActivity, DateRangeFilter } from '$lib/types';
	import { formatCompactNumber } from '$lib/types/usage';

	type Props = {
		data: DailyActivity[];
		dateRange: DateRangeFilter;
		onDateRangeChange: (range: DateRangeFilter) => void;
	};

	let { data, dateRange, onDateRangeChange }: Props = $props();

	type Metric = 'messageCount' | 'sessionCount' | 'toolCallCount';
	let metric = $state<Metric>('messageCount');

	const metricLabels: Record<Metric, string> = {
		messageCount: 'Messages',
		sessionCount: 'Sessions',
		toolCallCount: 'Tool Calls'
	};

	let tooltip = $state<{ x: number; y: number; label: string; value: number } | null>(null);

	const chartWidth = 800;
	const chartHeight = 300;
	const padding = { top: 20, right: 20, bottom: 40, left: 60 };
	const plotWidth = chartWidth - padding.left - padding.right;
	const plotHeight = chartHeight - padding.top - padding.bottom;

	const values = $derived(data.map((d) => d[metric]));
	const maxValue = $derived(Math.max(...values, 1));
	const barWidth = $derived(data.length > 0 ? Math.max(2, plotWidth / data.length - 2) : 10);

	function barX(i: number): number {
		if (data.length === 0) return 0;
		return padding.left + (i / data.length) * plotWidth + 1;
	}

	function barHeight(val: number): number {
		return (val / maxValue) * plotHeight;
	}

	function barY(val: number): number {
		return padding.top + plotHeight - barHeight(val);
	}

	function formatDateLabel(dateStr: string): string {
		const d = new Date(dateStr + 'T00:00:00');
		return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}

	// Show ~8 evenly spaced x-axis labels
	const labelIndices = $derived(() => {
		if (data.length <= 8) return data.map((_, i) => i);
		const step = Math.ceil(data.length / 8);
		const indices: number[] = [];
		for (let i = 0; i < data.length; i += step) indices.push(i);
		return indices;
	});

	// 5 gridlines
	const gridValues = $derived(
		[0.2, 0.4, 0.6, 0.8, 1.0].map((f) => Math.round(maxValue * f))
	);

	function handleBarHover(e: MouseEvent, i: number) {
		const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
		if (!rect) return;
		tooltip = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top - 10,
			label: formatDateLabel(data[i].date),
			value: data[i][metric]
		};
	}

	function handleBarLeave() {
		tooltip = null;
	}

	const ranges: DateRangeFilter[] = ['7d', '30d', 'all'];
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
>
	<div class="flex items-center justify-between mb-4">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Daily Activity</h3>
		<div class="flex items-center gap-2">
			<!-- Metric selector -->
			<div class="flex rounded-lg border border-gray-200 dark:border-gray-600 overflow-hidden">
				{#each (['messageCount', 'sessionCount', 'toolCallCount'] as const) as m}
					<button
						onclick={() => (metric = m)}
						class="px-2 py-1 text-xs font-medium transition-colors
							{metric === m
								? 'bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
								: 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'}"
					>
						{metricLabels[m]}
					</button>
				{/each}
			</div>
			<!-- Date range -->
			<div class="flex rounded-lg border border-gray-200 dark:border-gray-600 overflow-hidden">
				{#each ranges as r}
					<button
						onclick={() => onDateRangeChange(r)}
						class="px-2 py-1 text-xs font-medium transition-colors
							{dateRange === r
								? 'bg-primary-100 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
								: 'text-gray-500 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'}"
					>
						{r === 'all' ? 'All' : r}
					</button>
				{/each}
			</div>
		</div>
	</div>

	{#if data.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			No activity data available
		</div>
	{:else}
		<div class="relative">
			<svg viewBox="0 0 {chartWidth} {chartHeight}" class="w-full" preserveAspectRatio="xMidYMid meet">
				<!-- Gridlines -->
				{#each gridValues as gv}
					<line
						x1={padding.left}
						y1={barY(gv)}
						x2={chartWidth - padding.right}
						y2={barY(gv)}
						class="stroke-gray-200 dark:stroke-gray-700"
						stroke-width="1"
					/>
					<text
						x={padding.left - 8}
						y={barY(gv) + 4}
						text-anchor="end"
						class="fill-gray-400 dark:fill-gray-500"
						font-size="11"
					>
						{formatCompactNumber(gv)}
					</text>
				{/each}

				<!-- Bars -->
				{#each data as entry, i}
					<rect
						x={barX(i)}
						y={barY(entry[metric])}
						width={barWidth}
						height={barHeight(entry[metric])}
						class="fill-primary-500 dark:fill-primary-400 hover:fill-primary-600 dark:hover:fill-primary-300 transition-colors cursor-pointer"
						rx="1"
						role="img"
						onmouseenter={(e) => handleBarHover(e, i)}
						onmouseleave={handleBarLeave}
					/>
				{/each}

				<!-- X-axis labels -->
				{#each labelIndices() as li}
					<text
						x={barX(li) + barWidth / 2}
						y={chartHeight - 8}
						text-anchor="middle"
						class="fill-gray-400 dark:fill-gray-500"
						font-size="11"
					>
						{formatDateLabel(data[li].date)}
					</text>
				{/each}
			</svg>

			<!-- Tooltip -->
			{#if tooltip}
				<div
					class="absolute pointer-events-none bg-gray-900 dark:bg-gray-700 text-white text-xs rounded px-2 py-1 shadow-lg"
					style="left: {tooltip.x}px; top: {tooltip.y}px; transform: translate(-50%, -100%);"
				>
					{tooltip.label}: {formatCompactNumber(tooltip.value)}
				</div>
			{/if}
		</div>
	{/if}
</div>
