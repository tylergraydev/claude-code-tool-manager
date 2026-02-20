<script lang="ts">
	import { sessionStore } from '$lib/stores';
	import { estimateSessionCost, formatCost, formatCompactNumber } from '$lib/types/usage';
	import { Activity } from 'lucide-svelte';

	const totalSessions = $derived(
		sessionStore.projects.reduce((sum, p) => sum + p.sessionCount, 0)
	);

	const totalTokens = $derived(
		sessionStore.projects.reduce(
			(sum, p) => sum + p.totalInputTokens + p.totalOutputTokens,
			0
		)
	);

	const totalCost = $derived(
		sessionStore.projects.reduce((sum, p) => {
			return (
				sum +
				estimateSessionCost(
					p.modelsUsed,
					p.totalInputTokens,
					p.totalOutputTokens,
					p.totalCacheReadTokens,
					p.totalCacheCreationTokens
				)
			);
		}, 0)
	);

	const hasData = $derived(sessionStore.projects.length > 0);
</script>

{#if hasData}
	<div class="px-3 py-2.5 mx-1.5 mb-2 rounded-lg bg-gray-50 dark:bg-gray-700/30">
		<div class="flex items-center gap-1.5 mb-2">
			<Activity class="w-3 h-3 text-gray-400 dark:text-gray-500" />
			<span class="text-[10px] font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500">
				Usage
			</span>
			{#if sessionStore.isLoadingProjects}
				<div class="ml-auto w-1.5 h-1.5 rounded-full bg-primary-400 animate-pulse"></div>
			{/if}
		</div>
		<div class="grid grid-cols-3 gap-y-1">
			<div>
				<span class="text-[10px] text-gray-400 dark:text-gray-500">Projects</span>
				<p class="text-xs font-medium text-gray-700 dark:text-gray-300">
					{sessionStore.projects.length}
				</p>
			</div>
			<div>
				<span class="text-[10px] text-gray-400 dark:text-gray-500">Sessions</span>
				<p class="text-xs font-medium text-gray-700 dark:text-gray-300">
					{totalSessions}
				</p>
			</div>
			<div>
				<span class="text-[10px] text-gray-400 dark:text-gray-500">Cost</span>
				<p class="text-xs font-medium text-gray-700 dark:text-gray-300">
					{totalCost > 0 ? formatCost(totalCost) : '—'}
				</p>
			</div>
		</div>
		<p class="text-[10px] text-gray-400 dark:text-gray-500 mt-1.5">
			{formatCompactNumber(totalTokens)} tokens across all projects
		</p>
	</div>
{/if}
