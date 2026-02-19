<script lang="ts">
	import type { SessionFacet } from '$lib/types';
	import {
		OUTCOME_LABELS,
		OUTCOME_COLORS,
		HELPFULNESS_LABELS,
		HELPFULNESS_COLORS
	} from '$lib/types/insights';

	type Props = {
		facets: SessionFacet[];
	};

	let { facets }: Props = $props();

	const outcomeCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const f of facets) {
			if (f.outcome) counts[f.outcome] = (counts[f.outcome] || 0) + 1;
		}
		return Object.entries(counts).sort((a, b) => b[1] - a[1]);
	});

	const helpfulnessCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const f of facets) {
			if (f.claudeHelpfulness) counts[f.claudeHelpfulness] = (counts[f.claudeHelpfulness] || 0) + 1;
		}
		return Object.entries(counts).sort((a, b) => b[1] - a[1]);
	});

	const outcomeTotal = $derived(outcomeCounts.reduce((s, [, c]) => s + c, 0));
	const helpfulnessTotal = $derived(helpfulnessCounts.reduce((s, [, c]) => s + c, 0));

	// Donut chart helpers
	const cx = 80;
	const cy = 80;
	const radius = 55;
	const strokeWidth = 18;
	const circumference = 2 * Math.PI * radius;

	function makeSegments(
		entries: [string, number][],
		total: number,
		colors: Record<string, string>
	) {
		let offset = 0;
		return entries.map(([key, count]) => {
			const pct = total > 0 ? count / total : 0;
			const dash = circumference * pct;
			const seg = {
				key,
				count,
				pct,
				color: colors[key] || '#9ca3af',
				dashArray: `${dash} ${circumference - dash}`,
				dashOffset: -offset
			};
			offset += dash;
			return seg;
		});
	}

	const outcomeSegments = $derived(makeSegments(outcomeCounts, outcomeTotal, OUTCOME_COLORS));
	const helpfulnessSegments = $derived(
		makeSegments(helpfulnessCounts, helpfulnessTotal, HELPFULNESS_COLORS)
	);
</script>

<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
	<!-- Outcome Distribution -->
	<div
		class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
	>
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">
			Session Outcomes
		</h3>

		{#if outcomeCounts.length === 0}
			<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
				No outcome data available
			</div>
		{:else}
			<div class="flex flex-col items-center gap-4">
				<svg viewBox="0 0 160 160" class="w-40 h-40">
					{#each outcomeSegments as seg}
						<circle
							{cx}
							{cy}
							r={radius}
							fill="none"
							stroke={seg.color}
							stroke-width={strokeWidth}
							stroke-dasharray={seg.dashArray}
							stroke-dashoffset={seg.dashOffset}
							transform="rotate(-90 {cx} {cy})"
							class="transition-all duration-300"
						/>
					{/each}
					<text
						x={cx}
						y={cy - 4}
						text-anchor="middle"
						class="fill-gray-900 dark:fill-white"
						font-size="18"
						font-weight="bold"
					>
						{outcomeTotal}
					</text>
					<text
						x={cx}
						y={cy + 12}
						text-anchor="middle"
						class="fill-gray-400 dark:fill-gray-500"
						font-size="10"
					>
						sessions
					</text>
				</svg>

				<div class="flex flex-wrap justify-center gap-3">
					{#each outcomeSegments as seg}
						<div class="flex items-center gap-1.5">
							<div
								class="w-2.5 h-2.5 rounded-full"
								style="background: {seg.color}"
							></div>
							<span class="text-xs text-gray-600 dark:text-gray-300">
								{OUTCOME_LABELS[seg.key] || seg.key}
							</span>
							<span class="text-xs text-gray-400 dark:text-gray-500">
								{seg.count}
							</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>

	<!-- Helpfulness Distribution -->
	<div
		class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
	>
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">
			Claude Helpfulness
		</h3>

		{#if helpfulnessCounts.length === 0}
			<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
				No helpfulness data available
			</div>
		{:else}
			<div class="flex flex-col items-center gap-4">
				<svg viewBox="0 0 160 160" class="w-40 h-40">
					{#each helpfulnessSegments as seg}
						<circle
							{cx}
							{cy}
							r={radius}
							fill="none"
							stroke={seg.color}
							stroke-width={strokeWidth}
							stroke-dasharray={seg.dashArray}
							stroke-dashoffset={seg.dashOffset}
							transform="rotate(-90 {cx} {cy})"
							class="transition-all duration-300"
						/>
					{/each}
					<text
						x={cx}
						y={cy - 4}
						text-anchor="middle"
						class="fill-gray-900 dark:fill-white"
						font-size="18"
						font-weight="bold"
					>
						{helpfulnessTotal}
					</text>
					<text
						x={cx}
						y={cy + 12}
						text-anchor="middle"
						class="fill-gray-400 dark:fill-gray-500"
						font-size="10"
					>
						sessions
					</text>
				</svg>

				<div class="flex flex-wrap justify-center gap-3">
					{#each helpfulnessSegments as seg}
						<div class="flex items-center gap-1.5">
							<div
								class="w-2.5 h-2.5 rounded-full"
								style="background: {seg.color}"
							></div>
							<span class="text-xs text-gray-600 dark:text-gray-300">
								{HELPFULNESS_LABELS[seg.key] || seg.key}
							</span>
							<span class="text-xs text-gray-400 dark:text-gray-500">
								{seg.count}
							</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>
