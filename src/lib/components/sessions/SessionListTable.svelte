<script lang="ts">
	import type { SessionSummary, SessionSortField } from '$lib/types';
	import { totalTokens } from '$lib/types/session';
	import { formatCompactNumber, formatDuration, formatModelName, estimateSessionCost, formatCost } from '$lib/types/usage';
	import { ArrowUpDown, ArrowUp, ArrowDown } from 'lucide-svelte';

	type Props = {
		sessions: SessionSummary[];
		selectedSessionId: string | null;
		sortField: SessionSortField;
		sortDirection: 'asc' | 'desc';
		onSelectSession: (id: string) => void;
		onSort: (field: SessionSortField) => void;
	};

	let { sessions, selectedSessionId, sortField, sortDirection, onSelectSession, onSort }: Props =
		$props();

	function formatDate(iso: string | null): string {
		if (!iso) return 'N/A';
		try {
			return new Date(iso).toLocaleDateString(undefined, {
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return iso;
		}
	}

	function getSortIcon(field: SessionSortField) {
		if (sortField !== field) return ArrowUpDown;
		return sortDirection === 'asc' ? ArrowUp : ArrowDown;
	}

	function sessionCost(s: SessionSummary): number {
		return estimateSessionCost(
			s.modelsUsed,
			s.totalInputTokens,
			s.totalOutputTokens,
			s.totalCacheReadTokens,
			s.totalCacheCreationTokens
		);
	}

	const columns: { field: SessionSortField; label: string }[] = [
		{ field: 'date', label: 'Date' },
		{ field: 'duration', label: 'Duration' },
		{ field: 'messages', label: 'Messages' },
		{ field: 'tokens', label: 'Tokens' },
		{ field: 'cost', label: 'Est. Cost' }
	];
</script>

<div
	class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 overflow-hidden"
>
	<div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">
			Sessions ({sessions.length})
		</h3>
	</div>

	{#if sessions.length === 0}
		<div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500 text-sm">
			No sessions found
		</div>
	{:else}
		<div class="overflow-x-auto max-h-96 overflow-y-auto">
			<table class="w-full text-sm">
				<thead class="sticky top-0 bg-gray-50 dark:bg-gray-800/80 backdrop-blur">
					<tr>
						{#each columns as col}
							<th class="text-left px-4 py-2 font-medium text-gray-500 dark:text-gray-400">
								<button
									onclick={() => onSort(col.field)}
									class="flex items-center gap-1 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
								>
									{col.label}
									<svelte:component
										this={getSortIcon(col.field)}
										class="w-3.5 h-3.5 {sortField === col.field
											? 'text-primary-500'
											: 'text-gray-300 dark:text-gray-600'}"
									/>
								</button>
							</th>
						{/each}
						<th class="text-left px-4 py-2 font-medium text-gray-500 dark:text-gray-400"
							>Model</th
						>
						<th class="text-left px-4 py-2 font-medium text-gray-500 dark:text-gray-400"
							>Branch</th
						>
						<th class="text-left px-4 py-2 font-medium text-gray-500 dark:text-gray-400"
							>First Prompt</th
						>
					</tr>
				</thead>
				<tbody>
					{#each sessions as session}
						{@const isSelected = selectedSessionId === session.sessionId}
						<tr
							onclick={() => onSelectSession(session.sessionId)}
							class="cursor-pointer border-t border-gray-100 dark:border-gray-700/50 transition-colors
								{isSelected
								? 'bg-primary-50 dark:bg-primary-900/20'
								: 'hover:bg-gray-50 dark:hover:bg-gray-700/30'}"
						>
							<td class="px-4 py-2.5 whitespace-nowrap text-gray-900 dark:text-gray-100">
								{formatDate(session.firstTimestamp)}
							</td>
							<td class="px-4 py-2.5 whitespace-nowrap text-gray-600 dark:text-gray-400">
								{formatDuration(session.durationMs)}
							</td>
							<td class="px-4 py-2.5 whitespace-nowrap text-gray-600 dark:text-gray-400">
								{session.userMessageCount + session.assistantMessageCount}
							</td>
							<td class="px-4 py-2.5 whitespace-nowrap text-gray-600 dark:text-gray-400">
								{formatCompactNumber(totalTokens(session))}
							</td>
							<td class="px-4 py-2.5 whitespace-nowrap text-emerald-600 dark:text-emerald-400 font-medium">
								{formatCost(sessionCost(session))}
							</td>
							<td class="px-4 py-2.5 whitespace-nowrap">
								{#each session.modelsUsed as model}
									<span
										class="inline-block text-xs bg-violet-100 dark:bg-violet-900/30 text-violet-700 dark:text-violet-300 px-1.5 py-0.5 rounded mr-1"
									>
										{formatModelName(model)}
									</span>
								{/each}
							</td>
							<td class="px-4 py-2.5 whitespace-nowrap text-gray-600 dark:text-gray-400">
								{#if session.gitBranch}
									<span
										class="text-xs bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded font-mono"
									>
										{session.gitBranch}
									</span>
								{:else}
									<span class="text-gray-300 dark:text-gray-600">-</span>
								{/if}
							</td>
							<td
								class="px-4 py-2.5 max-w-xs truncate text-gray-500 dark:text-gray-400 text-xs"
							>
								{session.firstUserMessage ?? '-'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>
