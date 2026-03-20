<script lang="ts">
	type Props = {
		containerId: number;
	};

	let { containerId }: Props = $props();

	let stats = $state<any>(null);
	let isLoading = $state(true);
</script>

<div class="space-y-3">
	{#if isLoading}
		<p class="text-gray-500 dark:text-gray-400">Loading stats...</p>
	{:else if stats}
		<div class="grid grid-cols-2 gap-4">
			<div>
				<p class="text-sm text-gray-500">CPU</p>
				<p class="font-medium">{stats.cpuPercent}%</p>
			</div>
			<div>
				<p class="text-sm text-gray-500">Memory</p>
				<p class="font-medium">{stats.memoryUsage}</p>
			</div>
		</div>
	{:else}
		<p class="text-gray-500">No stats available</p>
	{/if}
</div>
