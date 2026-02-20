<script lang="ts">
	import type { DailyCost } from '$lib/stores/usageStore.svelte';
	import { getModelColor, formatModelName, formatCost } from '$lib/types/usage';

	type Props = {
		data: DailyCost[];
		models: string[];
	};

	let { data, models }: Props = $props();

	let tooltip = $state<{ x: number; y: number; label: string } | null>(null);

	const chartWidth = 800;
	const chartHeight = 300;
	const padding = { top: 20, right: 20, bottom: 40, left: 60 };
	const plotWidth = chartWidth - padding.left - padding.right;
	const plotHeight = chartHeight - padding.top - padding.bottom;

	const dayTotals = $derived(
		data.map((d) => d.total)
	);

	const maxValue = $derived(Math.max(...dayTotals, 0.01));
	const barWidth = $derived(data.length > 0 ? Math.max(2, plotWidth / data.length - 2) : 10);

	function barX(i: number): number {
		if (data.length === 0) return 0;
		return padding.left + (i / data.length) * plotWidth + 1;
	}

	function scaleY(val: number): number {
		return (val / maxValue) * plotHeight;
	}

	function formatDateLabel(dateStr: string): string {
		const d = new Date(dateStr + 'T00:00:00');
		return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}

	const labelIndices = $derived(() => {
		if (data.length <= 8) return data.map((_, i) => i);
		const step = Math.ceil(data.length / 8);
		const indices: number[] = [];
		for (let i = 0; i < data.length; i += step) indices.push(i);
		return indices;
	});

	const gridValues = $derived(
		[0.2, 0.4, 0.6, 0.8, 1.0].map((f) => maxValue * f)
	);

	function handleBarHover(e: MouseEvent, i: number) {
		const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
		if (!rect) return;
		const entry = data[i];
		const parts = models
			.map((m) => {
				const v = entry.costByModel[m] ?? 0;
				return v > 0 ? `${formatModelName(m)}: ${formatCost(v)}` : '';
			})
			.filter(Boolean)
			.join(', ');
		tooltip = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top - 10,
			label: `${formatDateLabel(entry.date)} — ${parts} (Total: ${formatCost(entry.total)})`
		};
	}

	function handleBarLeave() {
		tooltip = null;
	}
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
>
	<div class="flex items-center justify-between mb-4">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Daily Cost</h3>
		<!-- Legend -->
		<div class="flex flex-wrap gap-3">
			{#each models as m}
				<div class="flex items-center gap-1.5">
					<div class="w-2.5 h-2.5 rounded-full" style="background: {getModelColor(m)}"></div>
					<span class="text-xs text-gray-600 dark:text-gray-300">{formatModelName(m)}</span>
				</div>
			{/each}
		</div>
	</div>

	{#if data.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			No cost data available
		</div>
	{:else}
		<div class="relative">
			<svg viewBox="0 0 {chartWidth} {chartHeight}" class="w-full" preserveAspectRatio="xMidYMid meet">
				<!-- Gridlines -->
				{#each gridValues as gv}
					<line
						x1={padding.left}
						y1={padding.top + plotHeight - scaleY(gv)}
						x2={chartWidth - padding.right}
						y2={padding.top + plotHeight - scaleY(gv)}
						class="stroke-gray-200 dark:stroke-gray-700"
						stroke-width="1"
					/>
					<text
						x={padding.left - 8}
						y={padding.top + plotHeight - scaleY(gv) + 4}
						text-anchor="end"
						class="fill-gray-400 dark:fill-gray-500"
						font-size="11"
					>
						{formatCost(gv)}
					</text>
				{/each}

				<!-- Stacked bars -->
				{#each data as entry, i}
					{@const x = barX(i)}
					{#each models as m, mi}
						{@const val = entry.costByModel[m] ?? 0}
						{@const prevTotal = models.slice(0, mi).reduce((s, pm) => s + (entry.costByModel[pm] ?? 0), 0)}
						{#if val > 0}
							<rect
								{x}
								y={padding.top + plotHeight - scaleY(prevTotal + val)}
								width={barWidth}
								height={scaleY(val)}
								fill={getModelColor(m)}
								rx="1"
								class="cursor-pointer"
								role="img"
								onmouseenter={(e) => handleBarHover(e, i)}
								onmouseleave={handleBarLeave}
							/>
						{/if}
					{/each}
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
					class="absolute pointer-events-none bg-gray-900 dark:bg-gray-700 text-white text-xs rounded px-2 py-1 shadow-lg max-w-sm"
					style="left: {tooltip.x}px; top: {tooltip.y}px; transform: translate(-50%, -100%);"
				>
					{tooltip.label}
				</div>
			{/if}
		</div>
	{/if}
</div>
