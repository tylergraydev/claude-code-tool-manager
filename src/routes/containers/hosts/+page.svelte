<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { DockerHostList, DockerHostForm } from '$lib/components/containers';
	import { containerLibrary, notifications } from '$lib/stores';
	import type { CreateDockerHostRequest } from '$lib/types';
	import { Plus } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let showAddHost = $state(false);

	onMount(() => {
		containerLibrary.loadDockerHosts();
	});

	async function handleCreate(values: CreateDockerHostRequest) {
		try {
			await containerLibrary.createDockerHost(values);
			showAddHost = false;
			notifications.success('Docker host added');
		} catch (err) {
			notifications.error(`Failed to add host: ${err}`);
		}
	}
</script>

<Header
	title="Docker Hosts"
	subtitle="Manage Docker daemon connections (local and remote)"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex justify-end mb-6">
		<button onclick={() => showAddHost = true}
			class="btn btn-primary gap-2">
			<Plus class="w-4 h-4" aria-hidden="true" />
			Add Host
		</button>
	</div>

	<DockerHostList />
</div>

{#if showAddHost}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true"
		onkeydown={(e) => e.key === 'Escape' && (showAddHost = false)}
		onclick={(e) => e.target === e.currentTarget && (showAddHost = false)}>
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4">
			<div class="p-6">
				<h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Add Docker Host</h2>
				<DockerHostForm onSubmit={handleCreate} onCancel={() => showAddHost = false} />
			</div>
		</div>
	</div>
{/if}
