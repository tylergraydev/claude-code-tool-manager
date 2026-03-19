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
	let postCreateCommand = $state(container?.postCreateCommand || '');
	let postStartCommand = $state(container?.postStartCommand || '');
	let icon = $state(container?.icon || '');
	let tags = $state(container?.tags?.join(', ') || '');
	let ports = $state(container?.ports || []);
	let volumes = $state(container?.volumes || []);

	const isEditing = $derived(!!container?.id);
</script>

<form onsubmit={(e) => { e.preventDefault(); onSubmit({ name, description, image }); }}>
	<div class="space-y-4">
		<div>
			<label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Name *</label>
			<input id="name" bind:value={name} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Description</label>
			<input id="description" bind:value={description} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="image" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Image</label>
			<input id="image" bind:value={image} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="workingDir" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Working Directory</label>
			<input id="workingDir" bind:value={workingDir} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="dockerfile" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Dockerfile</label>
			<textarea id="dockerfile" bind:value={dockerfile} class="mt-1 block w-full rounded-md border-gray-300"></textarea>
		</div>

		<div>
			<label for="env" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Environment Variables</label>
		</div>

		<div>
			<label for="postCreate" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Post Create Command</label>
			<input id="postCreate" bind:value={postCreateCommand} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="postStart" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Post Start Command</label>
			<input id="postStart" bind:value={postStartCommand} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="icon" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Icon (emoji)</label>
			<input id="icon" bind:value={icon} class="mt-1 block w-full rounded-md border-gray-300" />
		</div>

		<div>
			<label for="tags" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Tags</label>
			<input id="tags" bind:value={tags} class="mt-1 block w-full rounded-md border-gray-300" />
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
