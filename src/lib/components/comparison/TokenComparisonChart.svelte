<script lang="ts">
	import type { ProjectComparisonData } from '$lib/stores/comparisonStore.svelte';
	import { formatCompactNumber } from '$lib/types/usage';

	type Props = {
		data: ProjectComparisonData[];
	};

	let { data }: Props = $props();

	const TOKEN_COLORS = {
		input: '#3b82f6',
		output: '#8b5cf6',
		cacheRead: '#10b981',
		cacheWrite: '#f59e0b'
	};

	const TOKEN_LABELS: Record<string, string> = {
		input: 'Input',
		output: 'Output',
		cacheRead: 'Cache Read',
		cacheWrite: 'Cache Write'
	};

	const chartWidth = 800;
	const chartHeight = 350;
	const padding = { top: 20, right: 20, bottom: 60, left: 70 };
	const plotWidth = chartWidth - padding.left - padding.right;
	const plotHeight = chartHeight - padding.top - padding.bottom;

	const tokenTypes = ['input', 'output', 'cacheRead', 'cacheWrite'] as const;

	function getTokenValue(d: ProjectComparisonData, type: string): number {
		switch (type) {
			case 'input': return d.project.totalInputTokens;
			case 'output': return d.project.totalOutputTokens;
			case 'cacheRead': return d.project.totalCacheReadTokens;
			case 'cacheWrite': return d.project.totalCacheCreationTokens;
			default: return 0;
		}
	}

	const maxValue = $derived.by(() => {
		let max = 1;
		for (const d of data) {
			for (const t of tokenTypes) {
				max = Math.max(max, getTokenValue(d, t));
			}
		}
		return max;
	});

	const gridValues = $derived(
		[0.2, 0.4, 0.6, 0.8, 1.0].map((f) => Math.round(maxValue * f))
	);

	function shortPath(path: string): string {
		const parts = path.split(/[/\\]/);
		return parts.slice(-1)[0] ?? path;
	}

	// Bar geometry
	const groupWidth = $derived(data.length > 0 ? plotWidth / data.length : 100);
	const barWidth = $derived(Math.max(4, (groupWidth - 20) / tokenTypes.length));

	function barX(groupIdx: number, barIdx: number): number {
		const groupStart = padding.left + groupIdx * groupWidth;
		const barsWidth = tokenTypes.length * barWidth;
		const groupOffset = (groupWidth - barsWidth) / 2;
		return groupStart + groupOffset + barIdx * barWidth;
	}

	function barY(val: number): number {
		return padding.top + plotHeight - (val / maxValue) * plotHeight;
	}

	function barHeight(val: number): number {
		return (val / maxValue) * plotHeight;
	}

	let tooltip = $state<{ x: number; y: number; label: string; value: string } | null>(null);

	function handleBarHover(e: MouseEvent, project: ProjectComparisonData, type: string) {
		const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
		if (!rect) return;
		const val = getTokenValue(project, type);
		tooltip = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top - 10,
			label: `${shortPath(project.project.inferredPath)} – ${TOKEN_LABELS[type]}`,
			value: formatCompactNumber(val)
		};
	}

	function handleBarLeave() {
		tooltip = null;
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4">
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-2">Token Comparison</h3>

	<!-- Legend -->
	<div class="flex flex-wrap gap-4 mb-3">
		{#each tokenTypes as type}
			<div class="flex items-center gap-1.5">
				<div class="w-3 h-3 rounded" style="background: {TOKEN_COLORS[type]}"></div>
				<span class="text-xs text-gray-600 dark:text-gray-400">{TOKEN_LABELS[type]}</span>
			</div>
		{/each}
	</div>

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
						{formatCompactNumber(gv)}
					</text>
				{/each}

				<!-- Bars -->
				{#each data as d, gi}
					{#each tokenTypes as type, bi}
						{@const val = getTokenValue(d, type)}
						<rect
							x={barX(gi, bi)}
							y={barY(val)}
							width={barWidth - 1}
							height={barHeight(val)}
							fill={TOKEN_COLORS[type]}
							rx="2"
							class="hover:opacity-80 transition-opacity cursor-pointer"
							role="img"
							onmouseenter={(e) => handleBarHover(e, d, type)}
							onmouseleave={handleBarLeave}
						/>
					{/each}

					<!-- X-axis label -->
					<text
						x={padding.left + gi * groupWidth + groupWidth / 2}
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
					{tooltip.label}: {tooltip.value}
				</div>
			{/if}
		</div>
	{/if}
</div>
