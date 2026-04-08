<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { TemplateBrowser, PortMappingEditor } from '$lib/components/containers';
	import { containerLibrary, notifications } from '$lib/stores';
	import type { ContainerTemplate, PortMapping } from '$lib/types';
	import { Plus, Trash2 } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let showConfigDialog = $state(false);
	let selectedTemplate = $state<ContainerTemplate | null>(null);
	let containerName = $state('');
	let ports = $state<PortMapping[]>([]);
	let envVars = $state<Array<{ key: string; value: string }>>([]);

	onMount(() => {
		containerLibrary.loadTemplates();
	});

	function handleUseTemplate(template: ContainerTemplate) {
		selectedTemplate = template;
		containerName = template.name.toLowerCase().replace(/[^a-z0-9]+/g, '-');
		ports = template.ports ? template.ports.map(p => ({ ...p })) : [];
		envVars = template.env ? Object.entries(template.env).map(([key, value]) => ({ key, value })) : [];
		showConfigDialog = true;
	}

	function addEnvVar() {
		envVars = [...envVars, { key: '', value: '' }];
	}

	function removeEnvVar(index: number) {
		envVars = envVars.filter((_, i) => i !== index);
	}

	async function handleCreate() {
		if (!selectedTemplate || !containerName) return;
		try {
			await containerLibrary.createFromTemplate(selectedTemplate.id, containerName);

			// Update the container with customized ports and env if changed
			const containers = containerLibrary.containers;
			const created = containers.find(c => c.name === containerName);
			if (created) {
				const env = envVars.length > 0
					? Object.fromEntries(envVars.filter(e => e.key).map(e => [e.key, e.value]))
					: undefined;
				await containerLibrary.update(created.id, {
					...created,
					ports: ports.length > 0 ? ports : undefined,
					env,
				});
			}

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

{#if showConfigDialog}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true">
		<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-lg w-full mx-4 p-6 max-h-[85vh] overflow-auto">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-1">
				Create from "{selectedTemplate?.name}"
			</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">Configure your container before creating.</p>

			<div class="space-y-4">
				<div>
					<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1 block">Container Name</label>
					<input
						type="text"
						bind:value={containerName}
						placeholder="container-name"
						class="input"
					/>
				</div>

				<PortMappingEditor bind:ports />

				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<label class="text-sm font-medium text-gray-700 dark:text-gray-300">Environment Variables</label>
						<button type="button" onclick={addEnvVar}
							class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400 flex items-center gap-1">
							<Plus class="w-3 h-3" aria-hidden="true" /> Add Variable
						</button>
					</div>
					{#if envVars.length > 0}
						<div class="space-y-2">
							<div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
								<span class="flex-1">Key</span>
								<span class="flex-1">Value</span>
								<span class="w-8"></span>
							</div>
							{#each envVars as env, i}
								<div class="flex items-center gap-2">
									<input type="text" bind:value={env.key} placeholder="KEY"
										class="input flex-1 py-1.5 font-mono text-sm" />
									<input type="text" bind:value={env.value} placeholder="value"
										class="input flex-1 py-1.5 font-mono text-sm" />
									<button type="button" onclick={() => removeEnvVar(i)} class="p-1 text-gray-400 hover:text-red-500">
										<Trash2 class="w-4 h-4" aria-hidden="true" />
									</button>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>

			<div class="flex justify-end gap-3 mt-6">
				<button onclick={() => showConfigDialog = false}
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
