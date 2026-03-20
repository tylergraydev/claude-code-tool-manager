<script lang="ts">
	type Props = {
		containerId: number;
	};

	let { containerId }: Props = $props();

	let logs = $state<string[]>([]);
	let autoScroll = $state(true);
	let tailLines = $state(100);
</script>

<div class="space-y-3">
	<div class="flex items-center gap-3">
		<label class="flex items-center gap-2 text-sm">
			<input type="checkbox" bind:checked={autoScroll} />
			Auto-scroll
		</label>
		<select bind:value={tailLines} class="text-sm rounded border-gray-300">
			<option value={50}>Last 50</option>
			<option value={100}>Last 100</option>
			<option value={500}>Last 500</option>
			<option value={1000}>Last 1000</option>
		</select>
	</div>

	<div class="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-200 h-64 overflow-auto">
		{#if logs.length === 0}
			<p class="text-gray-500">No logs available</p>
		{:else}
			{#each logs as line}
				<p>{line}</p>
			{/each}
		{/if}
	</div>
</div>
