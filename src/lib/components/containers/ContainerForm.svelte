<script lang="ts">
	import type { Container, CreateContainerRequest, PortMapping, VolumeMapping, ContainerType } from '$lib/types';
	import PortMappingEditor from './PortMappingEditor.svelte';
	import VolumeMappingEditor from './VolumeMappingEditor.svelte';

	let { container = null, onSubmit, onCancel }: {
		container?: Container | null;
		onSubmit: (values: CreateContainerRequest) => Promise<void> | void;
		onCancel: () => void;
	} = $props();

	let submitting = $state(false);

	let name = $state(container?.name || '');
	let description = $state(container?.description || '');
	let containerType = $state<ContainerType>(container?.containerType || 'docker');
	let image = $state(container?.image || '');
	let dockerfile = $state(container?.dockerfile || '');
	let devcontainerJson = $state(container?.devcontainerJson || '');
	let workingDir = $state(container?.workingDir || '/workspace');
	let postCreateCommand = $state(container?.postCreateCommand || '');
	let postStartCommand = $state(container?.postStartCommand || '');
	let icon = $state(container?.icon || '');
	let ports = $state<PortMapping[]>(container?.ports || []);
	let volumes = $state<VolumeMapping[]>(container?.volumes || []);
	let envText = $state(container?.env ? Object.entries(container.env).map(([k, v]) => `${k}=${v}`).join('\n') : '');
	let tagsText = $state(container?.tags?.join(', ') || '');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (submitting) return;
		submitting = true;
		try {
			const env: Record<string, string> = {};
			for (const line of envText.split('\n').filter(l => l.trim())) {
				const idx = line.indexOf('=');
				if (idx > 0) {
					env[line.substring(0, idx).trim()] = line.substring(idx + 1).trim();
				}
			}
			const tags = tagsText ? tagsText.split(',').map(t => t.trim()).filter(Boolean) : undefined;

			await onSubmit({
				name,
				description: description || undefined,
				containerType,
				image: image || undefined,
				dockerfile: dockerfile || undefined,
				devcontainerJson: containerType === 'devcontainer' ? (devcontainerJson || undefined) : undefined,
				workingDir: workingDir || undefined,
				postCreateCommand: postCreateCommand || undefined,
				postStartCommand: postStartCommand || undefined,
				icon: icon || undefined,
				ports: ports.length > 0 ? ports : undefined,
				volumes: volumes.length > 0 ? volumes : undefined,
				env: Object.keys(env).length > 0 ? env : undefined,
				tags
			});
		} finally {
			submitting = false;
		}
	}
</script>

<form onsubmit={handleSubmit} class="space-y-4">
	<div class="grid grid-cols-2 gap-4">
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name *</label>
			<input type="text" bind:value={name} required
				class="input" />
		</div>
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Type</label>
			<select bind:value={containerType}
				class="input">
				<option value="docker">Docker</option>
				<option value="devcontainer">Dev Container</option>
				<option value="custom">Custom</option>
			</select>
		</div>
	</div>

	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Description</label>
		<input type="text" bind:value={description}
			class="input" />
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Image</label>
			<input type="text" bind:value={image} placeholder="e.g. node:20-bookworm"
				class="input" />
		</div>
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Working Directory</label>
			<input type="text" bind:value={workingDir}
				class="input" />
		</div>
	</div>

	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Dockerfile</label>
		<textarea bind:value={dockerfile} rows={4} placeholder="FROM node:20-bookworm..."
			class="input font-mono"></textarea>
	</div>

	{#if containerType === 'devcontainer'}
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">devcontainer.json</label>
			<textarea bind:value={devcontainerJson} rows={6} placeholder={'{"image": "mcr.microsoft.com/devcontainers/base:ubuntu"}'}
				class="input font-mono"></textarea>
		</div>
	{/if}

	<PortMappingEditor bind:ports />
	<VolumeMappingEditor bind:volumes />

	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Environment Variables</label>
		<textarea bind:value={envText} rows={3} placeholder="KEY=value (one per line)"
			class="input font-mono"></textarea>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Post Create Command</label>
			<input type="text" bind:value={postCreateCommand}
				class="input" />
		</div>
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Post Start Command</label>
			<input type="text" bind:value={postStartCommand}
				class="input" />
		</div>
	</div>

	<div class="grid grid-cols-2 gap-4">
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Icon (emoji)</label>
			<input type="text" bind:value={icon} placeholder="📦"
				class="input" />
		</div>
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Tags</label>
			<input type="text" bind:value={tagsText} placeholder="node, web, dev"
				class="input" />
		</div>
	</div>

	<div class="flex justify-end gap-3 pt-2">
		<button type="button" onclick={onCancel}
			class="btn btn-secondary">
			Cancel
		</button>
		<button type="submit" disabled={!name || submitting}
			class="btn btn-primary">
			{#if submitting}
				Saving...
			{:else}
				{container ? 'Update' : 'Create'} Container
			{/if}
		</button>
	</div>
</form>
