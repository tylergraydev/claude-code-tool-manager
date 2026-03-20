<script lang="ts">
	import { containerLibrary } from '$lib/stores';
</script>

<div class="space-y-3">
	{#if containerLibrary.dockerHosts.length === 0}
		<p class="text-gray-500 dark:text-gray-400 text-sm">No Docker hosts configured</p>
	{:else}
		{#each containerLibrary.dockerHosts as host (host.id)}
			<div class="flex items-center justify-between p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
				<div>
					<div class="flex items-center gap-2">
						<span class="font-medium text-gray-900 dark:text-white">{host.name}</span>
						{#if host.isDefault}
							<span class="text-xs bg-primary-100 text-primary-600 dark:bg-primary-900/50 dark:text-primary-400 px-2 py-0.5 rounded">Default</span>
						{/if}
					</div>
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
						{host.hostType}{#if host.connectionUri} &middot; {host.connectionUri}{/if}
					</p>
				</div>
				<div class="flex items-center gap-1">
					<button aria-label="Test connection" onclick={() => containerLibrary.testDockerHost(host.id)} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-sm">
						Test
					</button>
					{#if host.id !== 1}
						<button aria-label="Delete host" onclick={() => containerLibrary.deleteDockerHost(host.id)} class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-red-500 text-sm">
							Delete
						</button>
					{/if}
				</div>
			</div>
		{/each}
	{/if}
</div>
