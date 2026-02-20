<script lang="ts">
	import type { ProjectComparisonData } from '$lib/stores/comparisonStore.svelte';
	import { formatCompactNumber } from '$lib/types/usage';

	type Props = {
		data: ProjectComparisonData[];
		tools: string[];
	};

	let { data, tools }: Props = $props();

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-1)[0] ?? path;
	}

	const labelWidth = 140;
	const chartWidth = 800;
	const padding = { top: 10, right: 20, bottom: 10, left: labelWidth };
	const plotWidth = chartWidth - padding.left - padding.right;
	const rowHeight = $derived(Math.max(24, 12 * data.length + 8));
	const chartHeight = $derived(padding.top + padding.bottom + tools.length * rowHeight);

	function getToolCount(d: ProjectComparisonData, tool: string): number {
		return d.project.toolUsage[tool] ?? 0;
	}

	const maxCount = $derived.by(() => {
		let max = 1;
		for (const tool of tools) {
			for (const d of data) {
				max = Math.max(max, getToolCount(d, tool));
			}
		}
		return max;
	});

	const barHeight = $derived(Math.max(6, Math.min(14, (rowHeight - 4) / data.length - 1)));

	function barYOffset(toolIdx: number, projectIdx: number): number {
		const rowTop = padding.top + toolIdx * rowHeight;
		const barsHeight = data.length * (barHeight + 1);
		const offset = (rowHeight - barsHeight) / 2;
		return rowTop + offset + projectIdx * (barHeight + 1);
	}

	function barW(val: number): number {
		return (val / maxCount) * plotWidth;
	}

	let tooltip = $state<{ x: number; y: number; label: string } | null>(null);

	function handleBarHover(e: MouseEvent, d: ProjectComparisonData, tool: string) {
		const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
		if (!rect) return;
		const count = getToolCount(d, tool);
		tooltip = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top - 10,
			label: `${shortPath(d.project.inferredPath)} – ${tool}: ${formatCompactNumber(count)}`
		};
	}

	function handleBarLeave() {
		tooltip = null;
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
	<div class="flex items-center justify-between mb-3">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Tool Usage (Top 10)</h3>
		<!-- Legend -->
		<div class="flex flex-wrap gap-3">
			{#each data as d}
				<div class="flex items-center gap-1.5">
					<div class="w-2.5 h-2.5 rounded-full" style="background: {d.color}"></div>
					<span class="text-xs text-gray-600 dark:text-gray-400">{shortPath(d.project.inferredPath)}</span>
				</div>
			{/each}
		</div>
	</div>

	{#if tools.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			No tool usage data
		</div>
	{:else}
		<div class="relative">
			<svg viewBox="0 0 {chartWidth} {chartHeight}" class="w-full" preserveAspectRatio="xMidYMid meet">
				{#each tools as tool, ti}
					<!-- Row background -->
					{#if ti % 2 === 0}
						<rect
							x="0"
							y={padding.top + ti * rowHeight}
							width={chartWidth}
							height={rowHeight}
							class="fill-gray-50 dark:fill-gray-800/50"
						/>
					{/if}

					<!-- Tool label -->
					<text
						x={labelWidth - 8}
						y={padding.top + ti * rowHeight + rowHeight / 2 + 4}
						text-anchor="end"
						class="fill-gray-600 dark:fill-gray-400"
						font-size="11"
					>
						{tool}
					</text>

					<!-- Bars per project -->
					{#each data as d, pi}
						{@const count = getToolCount(d, tool)}
						{#if count > 0}
							<rect
								x={padding.left}
								y={barYOffset(ti, pi)}
								width={barW(count)}
								height={barHeight}
								fill={d.color}
								rx="2"
								class="hover:opacity-80 transition-opacity cursor-pointer"
								role="img"
								onmouseenter={(e) => handleBarHover(e, d, tool)}
								onmouseleave={handleBarLeave}
							/>
						{/if}
					{/each}
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
