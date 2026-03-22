<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import type { ContainerLog } from '$lib/types';

	type Props = {
		containerId: number;
	};

	let { containerId }: Props = $props();

	let logs = $state<ContainerLog[]>([]);
	let autoScroll = $state(true);
	let tailLines = $state(100);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let logContainer: HTMLDivElement | undefined = $state();

	async function fetchLogs() {
		isLoading = true;
		error = null;
		try {
			logs = await containerLibrary.fetchLogs(containerId, tailLines, 0);
			if (autoScroll && logContainer) {
				requestAnimationFrame(() => {
					if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
				});
			}
		} catch (e) {
			error = String(e);
		} finally {
			isLoading = false;
		}
	}

	$effect(() => {
		// Re-fetch when containerId or tailLines changes
		const _id = containerId;
		const _tail = tailLines;
		fetchLogs();
	});
</script>

<div class="space-y-3">
	<div class="flex items-center gap-3">
		<label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
			<input type="checkbox" bind:checked={autoScroll} />
			Auto-scroll
		</label>
		<select bind:value={tailLines} aria-label="Number of log lines to show" class="input text-sm w-auto">
			<option value={50}>Last 50</option>
			<option value={100}>Last 100</option>
			<option value={500}>Last 500</option>
			<option value={1000}>Last 1000</option>
		</select>
		<button onclick={fetchLogs} class="btn btn-ghost text-sm" disabled={isLoading}>
			{isLoading ? 'Loading...' : 'Refresh'}
		</button>
	</div>

	{#if error}
		<div class="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm" role="alert">
			<span class="flex-1">Failed to load logs: {error}</span>
			<button onclick={fetchLogs} class="btn btn-ghost text-sm text-red-600 dark:text-red-400 px-2 py-1">Retry</button>
		</div>
	{/if}

	<div bind:this={logContainer} aria-label="Container log output" class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-200 h-64 overflow-auto">
		{#if isLoading}
			<p class="text-gray-500">Loading logs...</p>
		{:else if logs.length === 0 && !error}
			<p class="text-gray-500">No logs available. The container may not have produced any output, or it hasn't been started yet.</p>
		{:else}
			{#each logs as log}
				<p class={log.stream === 'stderr' ? 'text-red-400' : ''}>
					{#if log.timestamp}<span class="text-gray-500">{log.timestamp} </span>{/if}{log.message}
				</p>
			{/each}
		{/if}
	</div>
</div>
