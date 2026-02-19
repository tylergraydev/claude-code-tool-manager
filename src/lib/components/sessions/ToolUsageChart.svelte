<script lang="ts">
	import { formatCompactNumber } from '$lib/types/usage';

	type Props = {
		toolUsage: Record<string, number>;
	};

	let { toolUsage }: Props = $props();

	const sortedTools = $derived(
		Object.entries(toolUsage)
			.sort(([, a], [, b]) => b - a)
			.slice(0, 20)
	);

	const maxCount = $derived(sortedTools.length > 0 ? sortedTools[0][1] : 1);

	const barColors = [
		'#8b5cf6',
		'#3b82f6',
		'#10b981',
		'#f59e0b',
		'#ef4444',
		'#ec4899',
		'#14b8a6',
		'#6366f1',
		'#84cc16',
		'#f97316'
	];

	function barColor(index: number): string {
		return barColors[index % barColors.length];
	}

	const chartWidth = 600;
	const barHeight = 24;
	const labelWidth = 140;
	const countWidth = 60;
	const gap = 4;
	const plotWidth = chartWidth - labelWidth - countWidth - 20;

	const chartHeight = $derived(sortedTools.length * (barHeight + gap) + 10);
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 flex flex-col h-full"
>
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">Tool Usage</h3>

	{#if sortedTools.length === 0}
		<div class="flex items-center justify-center py-8 text-gray-400 dark:text-gray-500 text-sm">
			No tool usage data
		</div>
	{:else}
		<svg viewBox="0 0 {chartWidth} {chartHeight}" class="w-full" preserveAspectRatio="xMidYMid meet">
			{#each sortedTools as [name, count], i}
				{@const y = i * (barHeight + gap) + 5}
				{@const width = (count / maxCount) * plotWidth}

				<!-- Tool name label -->
				<text
					x={labelWidth - 8}
					y={y + barHeight / 2 + 4}
					text-anchor="end"
					class="fill-gray-600 dark:fill-gray-400"
					font-size="11"
				>
					{name.length > 18 ? name.substring(0, 16) + '...' : name}
				</text>

				<!-- Bar -->
				<rect
					x={labelWidth}
					{y}
					width={Math.max(width, 2)}
					height={barHeight}
					rx="3"
					fill={barColor(i)}
					opacity="0.8"
				/>

				<!-- Count label -->
				<text
					x={labelWidth + Math.max(width, 2) + 6}
					y={y + barHeight / 2 + 4}
					class="fill-gray-500 dark:fill-gray-400"
					font-size="11"
				>
					{formatCompactNumber(count)}
				</text>
			{/each}
		</svg>
	{/if}
</div>
