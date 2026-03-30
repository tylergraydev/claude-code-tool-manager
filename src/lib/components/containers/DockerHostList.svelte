<script lang="ts">
	import { containerLibrary, notifications } from '$lib/stores';
	import type { DockerHost } from '$lib/types';
	import { Server, Trash2, Wifi } from 'lucide-svelte';

	let testingHost = $state<number | null>(null);

	async function handleTest(host: DockerHost) {
		testingHost = host.id;
		try {
			const info = await containerLibrary.testDockerHost(host.hostType, host.connectionUri || undefined, host.sshKeyPath || undefined);
			notifications.success(`Connected: ${info}`);
		} catch (e) {
			notifications.error(`Connection failed: ${e}`);
		} finally {
			testingHost = null;
		}
	}

	async function handleDelete(host: DockerHost) {
		try {
			await containerLibrary.deleteDockerHost(host.id);
			notifications.success('Host removed');
		} catch (e) {
			notifications.error(String(e));
		}
	}
</script>

<div class="space-y-3">
	{#if containerLibrary.dockerHosts.length === 0}
		<div class="text-center py-12">
			<Server class="w-12 h-12 text-gray-300 dark:text-gray-600 mx-auto mb-3" aria-hidden="true" />
			<h3 class="text-lg font-medium text-gray-500 dark:text-gray-400">No Docker hosts configured</h3>
			<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">Add a Docker host to connect to remote Docker daemons</p>
		</div>
	{/if}
	{#each containerLibrary.dockerHosts as host (host.id)}
		<div class="card">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-3">
					<Server class="w-5 h-5 text-gray-400" aria-hidden="true" />
					<div>
						<div class="flex items-center gap-2">
							<h3 class="font-medium text-gray-900 dark:text-white">{host.name}</h3>
							{#if host.isDefault}
								<span class="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-400">Default</span>
							{/if}
						</div>
						<p class="text-sm text-gray-500 dark:text-gray-400">{host.hostType}{host.connectionUri ? ` — ${host.connectionUri}` : ''}</p>
					</div>
				</div>
				<div class="flex items-center gap-1">
					<button onclick={() => handleTest(host)} disabled={testingHost === host.id}
						class="p-1.5 rounded-lg text-gray-400 hover:text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20 transition-colors disabled:opacity-50"
						aria-label="Test connection">
						{#if testingHost === host.id}
							<div class="w-4 h-4 border-2 border-green-500 border-t-transparent rounded-full animate-spin"></div>
						{:else}
							<Wifi class="w-4 h-4" aria-hidden="true" />
						{/if}
					</button>
					{#if host.id !== 1}
						<button onclick={() => handleDelete(host)} class="p-1.5 rounded-lg text-gray-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors" aria-label="Delete host">
							<Trash2 class="w-4 h-4" aria-hidden="true" />
						</button>
					{/if}
				</div>
			</div>
		</div>
	{/each}
</div>
