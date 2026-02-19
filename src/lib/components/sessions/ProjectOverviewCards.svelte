<script lang="ts">
	import type { ProjectSummary } from '$lib/types';
	import { projectTotalTokens } from '$lib/types/session';
	import { formatCompactNumber, estimateSessionCost, formatCost } from '$lib/types/usage';
	import { FolderOpen, Hash, MessageSquare, Cpu, DollarSign } from 'lucide-svelte';

	type Props = {
		projects: ProjectSummary[];
	};

	let { projects }: Props = $props();

	const totalSessions = $derived(projects.reduce((sum, p) => sum + p.sessionCount, 0));
	const totalTokensAll = $derived(projects.reduce((sum, p) => sum + projectTotalTokens(p), 0));
	const totalModels = $derived(
		[...new Set(projects.flatMap((p) => p.modelsUsed))].length
	);
	const totalEstCost = $derived(
		projects.reduce(
			(sum, p) =>
				sum +
				estimateSessionCost(
					p.modelsUsed,
					p.totalInputTokens,
					p.totalOutputTokens,
					p.totalCacheReadTokens,
					p.totalCacheCreationTokens
				),
			0
		)
	);

	const cards = $derived([
		{
			label: 'Projects',
			value: projects.length.toString(),
			icon: FolderOpen,
			color: 'bg-blue-500'
		},
		{
			label: 'Total Sessions',
			value: formatCompactNumber(totalSessions),
			icon: Hash,
			color: 'bg-purple-500'
		},
		{
			label: 'Total Tokens',
			value: formatCompactNumber(totalTokensAll),
			icon: MessageSquare,
			color: 'bg-amber-500'
		},
		{
			label: 'Models Used',
			value: totalModels.toString(),
			icon: Cpu,
			color: 'bg-green-500'
		},
		{
			label: 'Est. API Cost',
			value: formatCost(totalEstCost),
			subtitle: 'if billed at API rates',
			icon: DollarSign,
			color: 'bg-emerald-500'
		}
	]);
</script>

<div class="grid grid-cols-2 lg:grid-cols-5 gap-3">
	{#each cards as card}
		<div
			class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4"
		>
			<div class="flex items-start justify-between">
				<div>
					<p class="text-sm font-medium text-gray-500 dark:text-gray-400">{card.label}</p>
					<p class="text-2xl font-bold text-gray-900 dark:text-white mt-0.5">{card.value}</p>
					{#if card.subtitle}
						<p class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">{card.subtitle}</p>
					{/if}
				</div>
				<div class="{card.color} p-2 rounded-lg">
					<card.icon class="w-4 h-4 text-white" />
				</div>
			</div>
		</div>
	{/each}
</div>
