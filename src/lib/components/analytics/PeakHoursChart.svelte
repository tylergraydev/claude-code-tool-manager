<script lang="ts">
	type Props = {
		hourCounts: number[];
	};

	let { hourCounts }: Props = $props();

	let tooltip = $state<{ x: number; y: number; hour: number; count: number } | null>(null);

	const chartWidth = 800;
	const chartHeight = 250;
	const padding = { top: 20, right: 20, bottom: 40, left: 50 };
	const plotWidth = chartWidth - padding.left - padding.right;
	const plotHeight = chartHeight - padding.top - padding.bottom;

	const maxCount = $derived(Math.max(...hourCounts, 1));
	const peakHour = $derived(hourCounts.indexOf(maxCount));
	const barWidth = $derived(plotWidth / 24 - 2);

	function barX(hour: number): number {
		return padding.left + (hour / 24) * plotWidth + 1;
	}

	function barHeight(count: number): number {
		return (count / maxCount) * plotHeight;
	}

	function barY(count: number): number {
		return padding.top + plotHeight - barHeight(count);
	}

	function barColor(hour: number, count: number): string {
		if (count === 0) return 'rgba(107, 114, 128, 0.2)';
		const intensity = count / maxCount;
		if (hour === peakHour) return '#8b5cf6';
		if (intensity > 0.7) return '#a78bfa';
		if (intensity > 0.4) return '#c4b5fd';
		return '#ddd6fe';
	}

	function barColorDark(hour: number, count: number): string {
		if (count === 0) return 'rgba(107, 114, 128, 0.15)';
		const intensity = count / maxCount;
		if (hour === peakHour) return '#8b5cf6';
		if (intensity > 0.7) return '#7c3aed';
		if (intensity > 0.4) return '#6d28d9';
		return '#5b21b6';
	}

	const hourLabels = $derived(
		Array.from({ length: 8 }, (_, i) => {
			const h = i * 3;
			if (h === 0) return { hour: h, label: '12am' };
			if (h === 12) return { hour: h, label: '12pm' };
			if (h < 12) return { hour: h, label: `${h}am` };
			return { hour: h, label: `${h - 12}pm` };
		})
	);

	const gridValues = $derived(
		[0.25, 0.5, 0.75, 1.0].map((f) => Math.round(maxCount * f))
	);

	function handleHover(e: MouseEvent, hour: number) {
		const rect = (e.currentTarget as SVGElement).closest('svg')?.getBoundingClientRect();
		if (!rect) return;
		tooltip = {
			x: e.clientX - rect.left,
			y: e.clientY - rect.top - 10,
			hour,
			count: hourCounts[hour]
		};
	}

	function handleLeave() {
		tooltip = null;
	}

	function formatHour(h: number): string {
		if (h === 0) return '12:00 AM';
		if (h === 12) return '12:00 PM';
		if (h < 12) return `${h}:00 AM`;
		return `${h - 12}:00 PM`;
	}

	// Use CSS media query to determine theme for bar colors
	// We'll use Tailwind classes instead of inline fill for theme support
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
>
	<div class="flex items-center justify-between mb-4">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Peak Hours</h3>
		{#if maxCount > 0}
			<span class="text-xs text-gray-400 dark:text-gray-500">
				Peak: {formatHour(peakHour)} ({maxCount} sessions)
			</span>
		{/if}
	</div>

	{#if hourCounts.every((c) => c === 0)}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			No hour data available
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
						{gv}
					</text>
				{/each}

				<!-- Bars (light mode) -->
				{#each hourCounts as count, hour}
					<rect
						x={barX(hour)}
						y={barY(count)}
						width={barWidth}
						height={barHeight(count)}
						rx="2"
						class="cursor-pointer dark:hidden"
						fill={barColor(hour, count)}
						role="img"
						onmouseenter={(e) => handleHover(e, hour)}
						onmouseleave={handleLeave}
					/>
					<rect
						x={barX(hour)}
						y={barY(count)}
						width={barWidth}
						height={barHeight(count)}
						rx="2"
						class="cursor-pointer hidden dark:block"
						fill={barColorDark(hour, count)}
						role="img"
						onmouseenter={(e) => handleHover(e, hour)}
						onmouseleave={handleLeave}
					/>
				{/each}

				<!-- X-axis labels -->
				{#each hourLabels as hl}
					<text
						x={barX(hl.hour) + barWidth / 2}
						y={chartHeight - 8}
						text-anchor="middle"
						class="fill-gray-400 dark:fill-gray-500"
						font-size="11"
					>
						{hl.label}
					</text>
				{/each}
			</svg>

			<!-- Tooltip -->
			{#if tooltip}
				<div
					class="absolute pointer-events-none bg-gray-900 dark:bg-gray-700 text-white text-xs rounded px-2 py-1 shadow-lg"
					style="left: {tooltip.x}px; top: {tooltip.y}px; transform: translate(-50%, -100%);"
				>
					{formatHour(tooltip.hour)}: {tooltip.count} sessions
				</div>
			{/if}
		</div>
	{/if}
</div>
