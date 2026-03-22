<script lang="ts">
	import PortMappingEditor from './PortMappingEditor.svelte';
	import VolumeMappingEditor from './VolumeMappingEditor.svelte';

	type Container = {
		id?: number;
		name?: string;
		description?: string;
		containerType?: string;
		image?: string;
		workingDir?: string;
		dockerfile?: string;
		repoUrl?: string;
		postCreateCommand?: string;
		postStartCommand?: string;
		icon?: string;
		tags?: string[];
		ports?: any[];
		volumes?: any[];
		env?: Record<string, string>;
	};

	type Props = {
		container?: Container;
		onSubmit: (container: any) => void;
		onCancel: () => void;
	};

	let { container, onSubmit, onCancel }: Props = $props();

	let name = $state(container?.name || '');
	let description = $state(container?.description || '');
	let image = $state(container?.image || '');
	let workingDir = $state(container?.workingDir || '');
	let dockerfile = $state(container?.dockerfile || '');
	let repoUrl = $state(container?.repoUrl || '');
	let postCreateCommand = $state(container?.postCreateCommand || '');
	let postStartCommand = $state(container?.postStartCommand || '');
	let icon = $state(container?.icon || '');
	let tags = $state(container?.tags?.join(', ') || '');
	let ports = $state(container?.ports || []);
	let volumes = $state(container?.volumes || []);
	let envEntries = $state<Array<{ key: string; value: string }>>(
		container?.env ? Object.entries(container.env).map(([key, value]) => ({ key, value })) : []
	);

	const isEditing = $derived(!!container?.id);

	function addEnvVar() {
		envEntries = [...envEntries, { key: '', value: '' }];
	}

	function removeEnvVar(index: number) {
		envEntries = envEntries.filter((_, i) => i !== index);
	}
</script>

<form onsubmit={(e) => {
	e.preventDefault();
	const parsedTags = tags ? tags.split(',').map((t: string) => t.trim()).filter(Boolean) : undefined;
	onSubmit({
		name,
		description: description || undefined,
		containerType: container?.containerType || 'docker',
		image: image || undefined,
		workingDir: workingDir || undefined,
		dockerfile: dockerfile || undefined,
		repoUrl: repoUrl || undefined,
		postCreateCommand: postCreateCommand || undefined,
		postStartCommand: postStartCommand || undefined,
		icon: icon || undefined,
		tags: parsedTags,
		ports: ports.length > 0 ? ports : undefined,
		volumes: volumes.length > 0 ? volumes : undefined,
		env: envEntries.length > 0 ? Object.fromEntries(envEntries.filter(e => e.key).map(e => [e.key, e.value])) : undefined,
	});
}}>
	<div class="space-y-4">
		<div>
			<label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name *</label>
			<input id="name" bind:value={name} required class="input w-full" />
		</div>

		<div>
			<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Description</label>
			<input id="description" bind:value={description} class="input w-full" />
		</div>

		<div>
			<label for="image" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Image</label>
			<input id="image" bind:value={image} class="input w-full" />
		</div>

		<div>
			<label for="workingDir" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Working Directory</label>
			<input id="workingDir" bind:value={workingDir} class="input w-full" />
		</div>

		<div>
			<label for="repoUrl" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Repository URL</label>
			<input id="repoUrl" bind:value={repoUrl} placeholder="https://github.com/user/repo" class="input w-full" />
			<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">Git repository to clone into the working directory on creation</p>
		</div>

		<div>
			<label for="dockerfile" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Dockerfile</label>
			<textarea id="dockerfile" bind:value={dockerfile} class="input w-full"></textarea>
		</div>

		<div>
			<div class="flex items-center justify-between mb-2">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Environment Variables</h4>
				<button type="button" onclick={addEnvVar} class="text-sm text-primary-600 hover:text-primary-700">
					Add Variable
				</button>
			</div>
			{#each envEntries as entry, i}
				<div class="flex items-center gap-2 mb-2">
					<input type="text" bind:value={entry.key} placeholder="KEY" class="input flex-1 text-sm font-mono" />
					<span class="text-gray-400">=</span>
					<input type="text" bind:value={entry.value} placeholder="value" class="input flex-1 text-sm font-mono" />
					<button type="button" onclick={() => removeEnvVar(i)} class="btn btn-ghost text-red-500 hover:text-red-700 text-sm px-2 py-1">Remove</button>
				</div>
			{/each}
		</div>

		<div>
			<label for="postCreate" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Post Create Command</label>
			<input id="postCreate" bind:value={postCreateCommand} class="input w-full" />
		</div>

		<div>
			<label for="postStart" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Post Start Command</label>
			<input id="postStart" bind:value={postStartCommand} class="input w-full" />
		</div>

		<div>
			<label for="icon" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Icon (emoji)</label>
			<input id="icon" bind:value={icon} class="input w-full" />
		</div>

		<div>
			<label for="tags" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Tags</label>
			<input id="tags" bind:value={tags} class="input w-full" />
		</div>

		<PortMappingEditor {ports} />
		<VolumeMappingEditor {volumes} />

		<div class="flex justify-end gap-2">
			<button type="button" onclick={onCancel} class="btn btn-secondary">Cancel</button>
			<button type="submit" class="btn btn-primary">
				{isEditing ? 'Update Container' : 'Create Container'}
			</button>
		</div>
	</div>
</form>
