<script lang="ts">
	import { containerLibrary } from '$lib/stores';
	import { ConfirmDialog } from '$lib/components/shared';

	let deletingHostId = $state<number | null>(null);
	let deletingHostName = $state('');

	function confirmDelete(host: { id: number; name: string }) {
		deletingHostId = host.id;
		deletingHostName = host.name;
	}

	async function handleDelete() {
		if (deletingHostId === null) return;
		try {
			await containerLibrary.deleteDockerHost(deletingHostId);
		} finally {
			deletingHostId = null;
		}
	}
</script>

<div class="space-y-3">
	{#if containerLibrary.dockerHosts.length === 0}
		<p class="text-gray-500 dark:text-gray-400 text-sm">No Docker hosts configured</p>
	{:else}
		{#each containerLibrary.dockerHosts as host (host.id)}
			<div class="flex items-center justify-between p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<span class="font-medium text-gray-900 dark:text-white truncate">{host.name}</span>
						{#if host.isDefault}
							<span class="text-xs bg-primary-100 text-primary-600 dark:bg-primary-900/50 dark:text-primary-400 px-2 py-0.5 rounded">Default</span>
						{/if}
					</div>
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">
						{host.hostType}{#if host.connectionUri} &middot; {host.connectionUri}{/if}
					</p>
				</div>
				<div class="flex items-center gap-1 shrink-0">
					<button aria-label="Test connection for {host.name}" onclick={() => containerLibrary.testDockerHost(host.hostType, host.connectionUri || '', host.sshKeyPath || '')} class="btn btn-ghost text-sm px-2 py-1">
						Test
					</button>
					{#if host.id !== 1}
						<button aria-label="Delete {host.name}" onclick={() => confirmDelete(host)} class="btn btn-ghost text-red-500 hover:text-red-700 text-sm px-2 py-1">
							Delete
						</button>
					{/if}
				</div>
			</div>
		{/each}
	{/if}
</div>

<ConfirmDialog
	open={deletingHostId !== null}
	title="Delete Docker Host"
	message="Are you sure you want to delete '{deletingHostName}'? Containers using this host may stop working."
	confirmText="Delete"
	onConfirm={handleDelete}
	onCancel={() => (deletingHostId = null)}
/>
