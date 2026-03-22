<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import type { ContainerStats as ContainerStatsType } from '$lib/types';
	import { onDestroy } from 'svelte';

	type Props = {
		containerId: number;
	};

	let { containerId }: Props = $props();

	let stats = $state<ContainerStatsType | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let refreshInterval: ReturnType<typeof setInterval> | null = null;

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	async function fetchStats() {
		try {
			stats = await containerLibrary.fetchStats(containerId);
			error = null;
		} catch (e) {
			error = String(e);
			stats = null;
		} finally {
			isLoading = false;
		}
	}

	$effect(() => {
		const _id = containerId;
		isLoading = true;
		fetchStats();

		refreshInterval = setInterval(fetchStats, 5000);

		return () => {
			if (refreshInterval) clearInterval(refreshInterval);
		};
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});
</script>

<div class="space-y-3">
	{#if isLoading}
		<p class="text-gray-500 dark:text-gray-400">Loading stats...</p>
	{:else if error}
		<p class="text-sm text-red-500">{error}</p>
	{:else if stats}
		<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
			<div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
				<p class="text-xs text-gray-500 dark:text-gray-400">CPU</p>
				<p class="text-lg font-semibold text-gray-900 dark:text-white">{stats.cpuPercent.toFixed(1)}%</p>
			</div>
			<div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
				<p class="text-xs text-gray-500 dark:text-gray-400">Memory</p>
				<p class="text-lg font-semibold text-gray-900 dark:text-white">{formatBytes(stats.memoryUsage)}</p>
				<p class="text-xs text-gray-400">{stats.memoryPercent.toFixed(1)}% of {formatBytes(stats.memoryLimit)}</p>
			</div>
			<div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
				<p class="text-xs text-gray-500 dark:text-gray-400">Network I/O</p>
				<p class="text-sm font-medium text-gray-900 dark:text-white">{formatBytes(stats.networkRxBytes)} / {formatBytes(stats.networkTxBytes)}</p>
			</div>
			<div class="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
				<p class="text-xs text-gray-500 dark:text-gray-400">PIDs</p>
				<p class="text-lg font-semibold text-gray-900 dark:text-white">{stats.pids}</p>
			</div>
		</div>
	{:else}
		<p class="text-gray-500 dark:text-gray-400">No stats available</p>
	{/if}
</div>
