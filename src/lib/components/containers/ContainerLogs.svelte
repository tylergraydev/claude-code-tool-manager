<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { containerLibrary } from '$lib/stores';
	import type { ContainerLog } from '$lib/types';

	let { containerId }: { containerId: number } = $props();

	let logs = $state<ContainerLog[]>([]);
	let tailLines = $state(100);
	let autoScroll = $state(true);
	let logContainer: HTMLDivElement;
	let interval: ReturnType<typeof setInterval> | null = null;

	async function fetchLogs() {
		try {
			logs = await containerLibrary.fetchLogs(containerId, tailLines);
			if (autoScroll && logContainer) {
				requestAnimationFrame(() => {
					logContainer.scrollTop = logContainer.scrollHeight;
				});
			}
		} catch {
			// Logs may be unavailable when container is not running
		}
	}

	onMount(() => {
		fetchLogs();
		interval = setInterval(fetchLogs, 1500);
	});

	onDestroy(() => {
		if (interval) clearInterval(interval);
	});
</script>

<div class="flex flex-col h-full">
	<div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-700">
		<div class="flex items-center gap-3">
			<select bind:value={tailLines} onchange={fetchLogs}
				class="input w-auto py-1 text-xs">
				<option value={50}>Last 50</option>
				<option value={100}>Last 100</option>
				<option value={500}>Last 500</option>
				<option value={1000}>Last 1000</option>
			</select>
		</div>
		<label class="flex items-center gap-1.5 text-xs text-gray-500">
			<input type="checkbox" bind:checked={autoScroll} class="rounded" />
			Auto-scroll
		</label>
	</div>
	<div bind:this={logContainer} class="flex-1 overflow-auto p-3 bg-gray-950 font-mono text-xs leading-5">
		{#if logs.length === 0}
			<p class="text-gray-500">No logs available</p>
		{:else}
			{#each logs as log}
				<div class="flex">
					{#if log.timestamp}
						<span class="text-gray-500 mr-2 shrink-0">{log.timestamp}</span>
					{/if}
					<span class="break-all {log.stream === 'stderr' ? 'text-red-400' : 'text-gray-300'}">{log.message}</span>
				</div>
			{/each}
		{/if}
	</div>
</div>
