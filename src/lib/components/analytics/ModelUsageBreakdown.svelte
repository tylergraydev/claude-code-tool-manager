<script lang="ts">
	import type { ModelUsageDetail } from '$lib/types';
	import { getModelColor, formatModelName, formatCompactNumber, estimateModelCost, formatCost } from '$lib/types/usage';

	type Props = {
		modelUsage: Record<string, ModelUsageDetail>;
	};

	let { modelUsage }: Props = $props();

	interface ModelEntry {
		id: string;
		name: string;
		color: string;
		total: number;
		detail: ModelUsageDetail;
	}

	const entries = $derived.by((): ModelEntry[] => {
		return Object.entries(modelUsage)
			.map(([id, detail]) => ({
				id,
				name: formatModelName(id),
				color: getModelColor(id),
				total:
					detail.inputTokens +
					detail.outputTokens +
					detail.cacheReadInputTokens +
					detail.cacheCreationInputTokens,
				detail
			}))
			.sort((a, b) => b.total - a.total);
	});

	const grandTotal = $derived(entries.reduce((sum, e) => sum + e.total, 0));

	// Donut chart geometry
	const cx = 100;
	const cy = 100;
	const radius = 70;
	const strokeWidth = 24;
	const circumference = 2 * Math.PI * radius;

	const segments = $derived.by(() => {
		let offset = 0;
		return entries.map((entry) => {
			const pct = grandTotal > 0 ? entry.total / grandTotal : 0;
			const dash = circumference * pct;
			const seg = { ...entry, dashArray: `${dash} ${circumference - dash}`, dashOffset: -offset, pct };
			offset += dash;
			return seg;
		});
	});
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
>
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">Model Usage</h3>

	{#if entries.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			No model usage data
		</div>
	{:else}
		<div class="flex flex-col items-center gap-4">
			<!-- Donut chart -->
			<svg viewBox="0 0 200 200" class="w-48 h-48">
				{#each segments as seg}
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
					y={cy - 6}
					text-anchor="middle"
					class="fill-gray-900 dark:fill-white"
					font-size="16"
					font-weight="bold"
				>
					{formatCompactNumber(grandTotal)}
				</text>
				<text
					x={cx}
					y={cy + 12}
					text-anchor="middle"
					class="fill-gray-400 dark:fill-gray-500"
					font-size="11"
				>
					total tokens
				</text>
			</svg>

			<!-- Legend -->
			<div class="flex flex-wrap justify-center gap-3">
				{#each segments as seg}
					<div class="flex items-center gap-1.5">
						<div class="w-2.5 h-2.5 rounded-full" style="background: {seg.color}"></div>
						<span class="text-xs text-gray-600 dark:text-gray-300">{seg.name}</span>
						<span class="text-xs text-gray-400 dark:text-gray-500">
							{(seg.pct * 100).toFixed(1)}%
						</span>
					</div>
				{/each}
			</div>

			<!-- Detail table -->
			<div class="w-full overflow-x-auto mt-2">
				<table class="w-full text-xs">
					<thead>
						<tr class="border-b border-gray-200 dark:border-gray-700">
							<th class="text-left py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Model
							</th>
							<th class="text-right py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Input
							</th>
							<th class="text-right py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Output
							</th>
							<th class="text-right py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Cache Read
							</th>
							<th class="text-right py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Cache Write
							</th>
							<th class="text-right py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Total
							</th>
							<th class="text-right py-2 px-1 font-medium text-gray-500 dark:text-gray-400">
								Est. Cost
							</th>
						</tr>
					</thead>
					<tbody>
						{#each entries as entry}
							<tr class="border-b border-gray-100 dark:border-gray-700/50">
								<td class="py-1.5 px-1">
									<div class="flex items-center gap-1.5">
										<div
											class="w-2 h-2 rounded-full"
											style="background: {entry.color}"
										></div>
										<span class="text-gray-700 dark:text-gray-300">{entry.name}</span>
									</div>
								</td>
								<td class="text-right py-1.5 px-1 text-gray-600 dark:text-gray-400">
									{formatCompactNumber(entry.detail.inputTokens)}
								</td>
								<td class="text-right py-1.5 px-1 text-gray-600 dark:text-gray-400">
									{formatCompactNumber(entry.detail.outputTokens)}
								</td>
								<td class="text-right py-1.5 px-1 text-gray-600 dark:text-gray-400">
									{formatCompactNumber(entry.detail.cacheReadInputTokens)}
								</td>
								<td class="text-right py-1.5 px-1 text-gray-600 dark:text-gray-400">
									{formatCompactNumber(entry.detail.cacheCreationInputTokens)}
								</td>
								<td class="text-right py-1.5 px-1 font-medium text-gray-900 dark:text-white">
									{formatCompactNumber(entry.total)}
								</td>
								<td class="text-right py-1.5 px-1 font-medium text-emerald-600 dark:text-emerald-400">
									{formatCost(
										entry.detail.costUSD > 0
											? entry.detail.costUSD
											: estimateModelCost(
													entry.id,
													entry.detail.inputTokens,
													entry.detail.outputTokens,
													entry.detail.cacheReadInputTokens,
													entry.detail.cacheCreationInputTokens
												)
									)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</div>
