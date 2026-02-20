<script lang="ts">
	import type { ProjectComparisonData } from '$lib/stores/comparisonStore.svelte';
	import { formatCost } from '$lib/types/usage';

	type Props = {
		data: ProjectComparisonData[];
	};

	let { data }: Props = $props();

	const chartWidth = 800;
	const chartHeight = 300;
	const padding = { top: 30, right: 20, bottom: 60, left: 70 };
	const plotWidth = chartWidth - padding.left - padding.right;
	const plotHeight = chartHeight - padding.top - padding.bottom;

	const maxCost = $derived(Math.max(...data.map((d) => d.estimatedCost), 0.01));

	const gridValues = $derived(
		[0.2, 0.4, 0.6, 0.8, 1.0].map((f) => maxCost * f)
	);

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-1)[0] ?? path;
	}

	const barWidth = $derived(data.length > 0 ? Math.min(80, (plotWidth / data.length) - 20) : 60);

	function barX(i: number): number {
		const groupWidth = plotWidth / data.length;
		return padding.left + i * groupWidth + (groupWidth - barWidth) / 2;
	}

	function barY(val: number): number {
		return padding.top + plotHeight - (val / maxCost) * plotHeight;
	}

	function barHeight(val: number): number {
		return (val / maxCost) * plotHeight;
	}

	let tooltip = $state<{ x: number; y: number; label: string } | null>(null);

	function handleBarHover(e: MouseEvent, d: ProjectComparisonData) {
		const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
		if (!rect) return;
		tooltip = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top - 10,
			label: `${shortPath(d.project.inferredPath)}: ${formatCost(d.estimatedCost)}`
		};
	}

	function handleBarLeave() {
		tooltip = null;
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">Cost Comparison</h3>

	{#if data.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			Select projects to compare
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
						{formatCost(gv)}
					</text>
				{/each}

				<!-- Bars -->
				{#each data as d, i}
					<rect
						x={barX(i)}
						y={barY(d.estimatedCost)}
						width={barWidth}
						height={barHeight(d.estimatedCost)}
						fill={d.color}
						rx="3"
						class="hover:opacity-80 transition-opacity cursor-pointer"
						role="img"
						onmouseenter={(e) => handleBarHover(e, d)}
						onmouseleave={handleBarLeave}
					/>

					<!-- Cost label on top -->
					<text
						x={barX(i) + barWidth / 2}
						y={barY(d.estimatedCost) - 6}
						text-anchor="middle"
						class="fill-gray-700 dark:fill-gray-300"
						font-size="11"
						font-weight="600"
					>
						{formatCost(d.estimatedCost)}
					</text>

					<!-- X-axis label -->
					<text
						x={barX(i) + barWidth / 2}
						y={chartHeight - 10}
						text-anchor="middle"
						class="fill-gray-500 dark:fill-gray-400"
						font-size="11"
					>
						{shortPath(d.project.inferredPath)}
					</text>
				{/each}
			</svg>

			<!-- Tooltip -->
			{#if tooltip}
				<div
					class="absolute pointer-events-none bg-gray-900 dark:bg-gray-700 text-white text-xs rounded px-2 py-1 shadow-lg z-10"
					style="left: {tooltip.x}px; top: {tooltip.y}px; transform: translate(-50%, -100%);"
				>
					{tooltip.label}
				</div>
			{/if}
		</div>
	{/if}
</div>
