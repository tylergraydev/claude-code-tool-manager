<script lang="ts">
	import ContainerLogs from './ContainerLogs.svelte';
	import ContainerStats from './ContainerStats.svelte';

	type Container = {
		id: number;
		name: string;
		description?: string;
		containerType: string;
		image?: string;
		icon?: string;
		ports?: { hostPort: number; containerPort: number; protocol: string }[];
		volumes?: any[];
		env?: Record<string, string>;
		workingDir?: string;
		dockerContainerId?: string;
	};

	type Props = {
		container: Container;
		onClose: () => void;
	};

	let { container, onClose }: Props = $props();

	let activeTab = $state('overview');

	const typeLabels: Record<string, string> = {
		docker: 'Docker',
		devcontainer: 'Dev Container',
		custom: 'Custom'
	};
</script>

<div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-6">
	<div class="flex items-start justify-between mb-4">
		<div>
			<h2 class="text-lg font-semibold text-gray-900 dark:text-white">{container.name}</h2>
			{#if container.description}
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{container.description}</p>
			{/if}
		</div>
		<button aria-label="Close" onclick={onClose} class="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
			X
		</button>
	</div>

	<div class="flex gap-2 mb-4 border-b border-gray-200 dark:border-gray-700">
		<button
			class="px-3 py-2 text-sm font-medium"
			class:text-primary-600={activeTab === 'overview'}
			onclick={() => activeTab = 'overview'}
		>Overview</button>
		<button
			class="px-3 py-2 text-sm font-medium"
			class:text-primary-600={activeTab === 'logs'}
			onclick={() => activeTab = 'logs'}
		>Logs</button>
		<button
			class="px-3 py-2 text-sm font-medium"
			class:text-primary-600={activeTab === 'stats'}
			onclick={() => activeTab = 'stats'}
		>Stats</button>
		<button
			class="px-3 py-2 text-sm font-medium"
			class:text-primary-600={activeTab === 'exec'}
			onclick={() => activeTab = 'exec'}
		>Exec</button>
	</div>

	{#if activeTab === 'overview'}
		<div class="space-y-3">
			<div class="grid grid-cols-2 gap-3 text-sm">
				<div>
					<span class="text-gray-500">Type:</span>
					<span class="ml-2 font-medium">{typeLabels[container.containerType] || container.containerType}</span>
				</div>
				<div>
					<span class="text-gray-500">Image:</span>
					<span class="ml-2 font-medium font-mono">{container.image || 'N/A'}</span>
				</div>
				<div>
					<span class="text-gray-500">Working Dir:</span>
					<span class="ml-2 font-medium font-mono">{container.workingDir || 'N/A'}</span>
				</div>
				<div>
					<span class="text-gray-500">Container ID:</span>
					<span class="ml-2 font-medium font-mono">{container.dockerContainerId || 'N/A'}</span>
				</div>
			</div>

			{#if container.ports && container.ports.length > 0}
				<div>
					<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Ports</h4>
					{#each container.ports as port}
						<span class="text-sm font-mono">{port.hostPort}:{port.containerPort}/{port.protocol}</span>
					{/each}
				</div>
			{/if}

			{#if container.env && Object.keys(container.env).length > 0}
				<div>
					<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Environment</h4>
					{#each Object.entries(container.env) as [key, value]}
						<div class="text-sm font-mono">
							<span class="text-gray-600 dark:text-gray-400">{key}</span>
							<span class="text-gray-400 mx-1">=</span>
							<span>{value}</span>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{:else if activeTab === 'logs'}
		<ContainerLogs containerId={container.id} />
	{:else if activeTab === 'stats'}
		<ContainerStats containerId={container.id} />
	{:else if activeTab === 'exec'}
		<div class="text-gray-500 dark:text-gray-400">Exec terminal</div>
	{/if}
</div>
