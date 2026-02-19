<script lang="ts">
	import type { SessionFacet } from '$lib/types';
	import { FRICTION_LABELS } from '$lib/types/insights';

	type Props = {
		facets: SessionFacet[];
	};

	let { facets }: Props = $props();

	const frictionCounts = $derived.by(() => {
		const counts: Record<string, number> = {};
		for (const f of facets) {
			for (const [key, value] of Object.entries(f.frictionCounts)) {
				counts[key] = (counts[key] || 0) + value;
			}
		}
		return Object.entries(counts).sort((a, b) => b[1] - a[1]);
	});

	const maxCount = $derived(
		frictionCounts.length > 0 ? Math.max(...frictionCounts.map(([, c]) => c)) : 0
	);

	const barColors = [
		'#ef4444', '#f59e0b', '#3b82f6', '#8b5cf6',
		'#10b981', '#ec4899', '#06b6d4', '#f97316'
	];
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
>
	<h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-4">
		Friction Categories
	</h3>

	{#if frictionCounts.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
			No friction data recorded
		</div>
	{:else}
		<div class="space-y-3">
			{#each frictionCounts as [key, count], i}
				{@const pct = maxCount > 0 ? (count / maxCount) * 100 : 0}
				{@const color = barColors[i % barColors.length]}
				<div>
					<div class="flex items-center justify-between mb-1">
						<span class="text-xs font-medium text-gray-600 dark:text-gray-300">
							{FRICTION_LABELS[key] || key}
						</span>
						<span class="text-xs text-gray-400 dark:text-gray-500">
							{count}
						</span>
					</div>
					<div class="w-full h-5 bg-gray-100 dark:bg-gray-700 rounded-full overflow-hidden">
						<div
							class="h-full rounded-full transition-all duration-500"
							style="width: {pct}%; background: {color};"
						></div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
