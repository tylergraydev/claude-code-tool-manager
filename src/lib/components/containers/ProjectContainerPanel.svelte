<script lang="ts">
	import { containerLibrary } from '$lib/stores';

	type Props = {
		projectId: number;
	};

	let { projectId }: Props = $props();

	let containers = $state<any[]>([]);
	let isLoading = $state(false);
</script>

<div class="space-y-3">
	<div class="flex items-center justify-between">
		<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Containers</h3>
		<button class="text-sm text-primary-600 hover:text-primary-700">Assign</button>
	</div>

	{#if containers.length === 0}
		<p class="text-sm text-gray-500 dark:text-gray-400">No containers assigned</p>
	{:else}
		{#each containers as container (container.id)}
			<div class="p-2 bg-gray-50 dark:bg-gray-700 rounded text-sm">
				{container.name}
			</div>
		{/each}
	{/if}
</div>
