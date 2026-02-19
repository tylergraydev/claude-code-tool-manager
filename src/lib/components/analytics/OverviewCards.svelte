<script lang="ts">
	import { Hash, MessageSquare, Wrench, Calendar, Clock } from 'lucide-svelte';
	import type { LongestSession } from '$lib/types';
	import { formatCompactNumber, formatDuration } from '$lib/types/usage';

	type Props = {
		totalSessions: number | null;
		totalMessages: number | null;
		totalToolCalls: number;
		firstSessionDate: string | null;
		longestSession: LongestSession | null;
		lastComputedDate: string | null;
	};

	let {
		totalSessions,
		totalMessages,
		totalToolCalls,
		firstSessionDate,
		longestSession,
		lastComputedDate
	}: Props = $props();

	function formatDate(iso: string | null): string {
		if (!iso) return 'N/A';
		try {
			return new Date(iso).toLocaleDateString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return iso;
		}
	}

	const cards = $derived([
		{
			label: 'Total Sessions',
			value: totalSessions != null ? formatCompactNumber(totalSessions) : '0',
			icon: Hash,
			color: 'bg-blue-500'
		},
		{
			label: 'Total Messages',
			value: totalMessages != null ? formatCompactNumber(totalMessages) : '0',
			icon: MessageSquare,
			color: 'bg-purple-500'
		},
		{
			label: 'Tool Calls',
			value: formatCompactNumber(totalToolCalls),
			icon: Wrench,
			color: 'bg-amber-500'
		},
		{
			label: 'First Session',
			value: formatDate(firstSessionDate),
			icon: Calendar,
			color: 'bg-green-500'
		},
		{
			label: 'Longest Session',
			value: longestSession ? formatDuration(longestSession.duration) : 'N/A',
			subtitle: longestSession ? `${longestSession.messageCount} messages` : undefined,
			icon: Clock,
			color: 'bg-rose-500'
		}
	]);
</script>

<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3">
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

{#if lastComputedDate}
	<p class="text-xs text-gray-400 dark:text-gray-500 mt-2">
		Last updated: {formatDate(lastComputedDate)}
	</p>
{/if}
