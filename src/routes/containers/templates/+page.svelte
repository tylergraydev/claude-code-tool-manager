<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { TemplateBrowser } from '$lib/components/containers';
	import { containerLibrary, notifications } from '$lib/stores';
	import type { ContainerTemplate } from '$lib/types';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let showNameDialog = $state(false);
	let selectedTemplate = $state<ContainerTemplate | null>(null);
	let containerName = $state('');

	onMount(() => {
		containerLibrary.loadTemplates();
	});

	function handleUseTemplate(template: ContainerTemplate) {
		selectedTemplate = template;
		containerName = template.name.toLowerCase().replace(/[^a-z0-9]+/g, '-');
		showNameDialog = true;
	}

	async function handleCreate() {
		if (!selectedTemplate || !containerName) return;
		try {
			await containerLibrary.createFromTemplate(selectedTemplate.id, containerName);
			notifications.success(`Container '${containerName}' created from template`);
			goto('/containers');
		} catch (err) {
			notifications.error(`Failed to create container: ${err}`);
		}
	}
</script>

<Header
	title="Container Templates"
	subtitle="Quick-start templates for common development environments"
/>

<div class="flex-1 overflow-auto p-6">
	<TemplateBrowser onUse={handleUseTemplate} />
</div>

{#if showNameDialog}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true"
		onkeydown={(e) => e.key === 'Escape' && (showNameDialog = false)}
		onclick={(e) => e.target === e.currentTarget && (showNameDialog = false)}>
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-md w-full mx-4 p-6">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
				Create from "{selectedTemplate?.name}"
			</h3>
			<p class="text-sm text-gray-500 mb-4">Choose a name for your new container.</p>
			<input
				type="text"
				bind:value={containerName}
				placeholder="container-name"
				class="input mb-4"
			/>
			<div class="flex justify-end gap-3">
				<button onclick={() => showNameDialog = false}
					class="btn btn-secondary">
					Cancel
				</button>
				<button onclick={handleCreate} disabled={!containerName}
					class="btn btn-primary">
					Create Container
				</button>
			</div>
		</div>
	</div>
{/if}
