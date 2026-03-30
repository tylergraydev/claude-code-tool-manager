<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { containerLibrary } from '$lib/stores';
	import type { ContainerStats as Stats } from '$lib/types';

	let { containerId }: { containerId: number } = $props();

	let stats = $state<Stats | null>(null);
	let error = $state<string | null>(null);
	let interval: ReturnType<typeof setInterval> | null = null;

	function formatBytes(bytes: number): string {
		if (!bytes || bytes <= 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.min(Math.floor(Math.log(bytes) / Math.log(k)), sizes.length - 1);
		return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
	}

	async function fetchStats() {
		try {
			stats = await containerLibrary.fetchStats(containerId);
			error = null;
		} catch (e) {
			error = String(e);
		}
	}

	onMount(() => {
		fetchStats();
		interval = setInterval(fetchStats, 3000);
	});

	onDestroy(() => {
		if (interval) clearInterval(interval);
	});
</script>

<div class="p-4 space-y-4">
	{#if error}
		<p class="text-sm text-red-500">{error}</p>
	{:else if !stats}
		<p class="text-sm text-gray-500">Loading stats...</p>
	{:else}
		<div class="grid grid-cols-2 gap-4">
			<div>
				<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">CPU Usage</p>
				<div class="flex items-center gap-2">
					<div class="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
						<div class="h-full bg-blue-500 rounded-full transition-all" style="width: {Math.min(stats.cpuPercent, 100)}%"></div>
					</div>
					<span class="text-sm font-medium text-gray-700 dark:text-gray-300 w-14 text-right">{stats.cpuPercent.toFixed(1)}%</span>
				</div>
			</div>
			<div>
				<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Memory</p>
				<div class="flex items-center gap-2">
					<div class="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
						<div class="h-full bg-green-500 rounded-full transition-all" style="width: {Math.min(stats.memoryPercent, 100)}%"></div>
					</div>
					<span class="text-sm font-medium text-gray-700 dark:text-gray-300 w-14 text-right">{stats.memoryPercent.toFixed(1)}%</span>
				</div>
				<p class="text-xs text-gray-400 mt-0.5">{formatBytes(stats.memoryUsage)} / {formatBytes(stats.memoryLimit)}</p>
			</div>
			<div>
				<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Network I/O</p>
				<p class="text-sm text-gray-700 dark:text-gray-300">{formatBytes(stats.networkRxBytes)} / {formatBytes(stats.networkTxBytes)}</p>
			</div>
			<div>
				<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">PIDs</p>
				<p class="text-sm text-gray-700 dark:text-gray-300">{stats.pids}</p>
			</div>
		</div>
	{/if}
</div>
