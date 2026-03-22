<script lang="ts">
	import ContainerLogs from './ContainerLogs.svelte';
	import ContainerStats from './ContainerStats.svelte';
	import ContainerTerminal from './ContainerTerminal.svelte';
	import ContainerFiles from './ContainerFiles.svelte';
	import ContainerStatus from './ContainerStatus.svelte';
	import { containerLibrary } from '$lib/stores';
	import { notifications } from '$lib/stores/notifications.svelte';
	import { Play, Square, RotateCw, Copy, Terminal } from 'lucide-svelte';

	type Container = {
		id: number;
		name: string;
		description?: string;
		containerType: string;
		image?: string;
		icon?: string;
		ports?: { hostPort: number; containerPort: number; protocol?: string }[];
		volumes?: any[];
		env?: Record<string, string>;
		workingDir?: string;
		dockerContainerId?: string;
	};

	type Props = {
		container: Container;
		status?: string;
		initialTab?: string;
		onClose: () => void;
	};

	let { container, status, initialTab = 'overview', onClose }: Props = $props();

	let activeTab = $state(initialTab);
	let actionError = $state<string | null>(null);

	const typeLabels: Record<string, string> = {
		docker: 'Docker',
		devcontainer: 'Dev Container',
		custom: 'Custom'
	};

	const tabs = ['overview', 'logs', 'stats', 'console', 'files'];

	const isRunning = $derived(status === 'running');

	async function handleStart() {
		actionError = null;
		try { await containerLibrary.startContainer(container.id); } catch (e) { actionError = String(e); }
	}

	async function handleStop() {
		actionError = null;
		try { await containerLibrary.stopContainer(container.id); } catch (e) { actionError = String(e); }
	}

	async function handleRestart() {
		actionError = null;
		try { await containerLibrary.restartContainer(container.id); } catch (e) { actionError = String(e); }
	}

	const containerSlug = $derived('cctm-' + container.name.toLowerCase().replace(/\s+/g, '-'));
	const dockerExecCmd = $derived(`docker exec -it ${containerSlug} bash`);
	const claudeCmd = $derived(`docker exec -it ${containerSlug} bash -c "npx @anthropic-ai/claude-code"`);

	let copiedField = $state<string | null>(null);

	async function copyToClipboard(text: string, label: string) {
		try {
			await navigator.clipboard.writeText(text);
			copiedField = label;
			notifications.success(`Copied ${label} command`);
			setTimeout(() => { if (copiedField === label) copiedField = null; }, 2000);
		} catch {
			notifications.error('Failed to copy to clipboard');
		}
	}
</script>

<div class="flex items-start justify-between mb-4">
	<div class="min-w-0">
		<div class="flex items-center gap-2 flex-wrap">
			<h2 class="text-lg font-semibold text-gray-900 dark:text-white">{container.name}</h2>
			{#if status}
				<ContainerStatus {status} />
			{/if}
		</div>
		{#if container.description}
			<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">{container.description}</p>
		{/if}
	</div>
	<div class="flex items-center gap-1 shrink-0">
		<!-- Lifecycle buttons -->
		{#if isRunning}
			<button onclick={handleStop} class="btn btn-ghost p-2 text-red-500" title="Stop" aria-label="Stop container">
				<Square class="w-4 h-4" />
			</button>
			<button onclick={handleRestart} class="btn btn-ghost p-2 text-gray-500 dark:text-gray-400" title="Restart" aria-label="Restart container">
				<RotateCw class="w-4 h-4" />
			</button>
		{:else}
			<button onclick={handleStart} class="btn btn-ghost p-2 text-green-600 dark:text-green-400" title="Start" aria-label="Start container">
				<Play class="w-4 h-4" />
			</button>
		{/if}

		<button aria-label="Close" onclick={onClose} class="btn btn-ghost p-2">
			<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
		</button>
	</div>
</div>

{#if actionError}
	<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-red-700 dark:text-red-400 text-sm" role="alert">
		{actionError}
	</div>
{/if}

<div class="flex gap-1 mb-4 border-b border-gray-200 dark:border-gray-700" role="tablist">
	{#each tabs as tab}
		<button
			role="tab"
			aria-selected={activeTab === tab}
			aria-controls="panel-{tab}"
			class="px-3 py-2 text-sm font-medium -mb-px border-b-2 transition-colors {activeTab === tab
				? 'border-primary-600 text-primary-600 dark:border-primary-400 dark:text-primary-400'
				: 'border-transparent text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'}"
			onclick={() => activeTab = tab}
		>{tab.charAt(0).toUpperCase() + tab.slice(1)}</button>
	{/each}
</div>

{#if activeTab === 'overview'}
	<div id="panel-overview" role="tabpanel" class="space-y-3">
		<div class="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm">
			<div>
				<span class="text-gray-500 dark:text-gray-400">Type:</span>
				<span class="ml-2 font-medium text-gray-900 dark:text-white">{typeLabels[container.containerType] || container.containerType}</span>
			</div>
			<div>
				<span class="text-gray-500 dark:text-gray-400">Image:</span>
				<span class="ml-2 font-medium font-mono text-gray-900 dark:text-white">{container.image || 'N/A'}</span>
			</div>
			<div>
				<span class="text-gray-500 dark:text-gray-400">Working Dir:</span>
				<span class="ml-2 font-medium font-mono text-gray-900 dark:text-white">{container.workingDir || 'N/A'}</span>
			</div>
			<div>
				<span class="text-gray-500 dark:text-gray-400">Container ID:</span>
				<span class="ml-2 font-medium font-mono text-gray-900 dark:text-white">{container.dockerContainerId ? container.dockerContainerId.slice(0, 12) : 'N/A'}</span>
			</div>
		</div>

		{#if container.ports && container.ports.length > 0}
			<div>
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Ports</h4>
				<div class="flex flex-wrap gap-2">
					{#each container.ports as port}
						<span class="text-sm font-mono bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded">{port.hostPort}:{port.containerPort}/{port.protocol || 'tcp'}</span>
					{/each}
				</div>
			</div>
		{/if}

		{#if container.env && Object.keys(container.env).length > 0}
			<div>
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Environment</h4>
				{#each Object.entries(container.env) as [key, value]}
					<div class="text-sm font-mono">
						<span class="text-gray-600 dark:text-gray-400">{key}</span>
						<span class="text-gray-400 mx-1">=</span>
						<span class="text-gray-900 dark:text-white">{value}</span>
					</div>
				{/each}
			</div>
		{/if}

		{#if isRunning}
			<div class="border-t border-gray-200 dark:border-gray-700 pt-3 mt-3">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Connect</h4>
				<div class="space-y-2">
					<div class="flex items-center gap-2">
						<div class="flex-1 bg-gray-900 rounded-lg px-3 py-2 font-mono text-sm text-gray-200 truncate">
							<Terminal class="w-3.5 h-3.5 inline mr-1.5 text-green-400" />{dockerExecCmd}
						</div>
						<button onclick={() => copyToClipboard(dockerExecCmd, 'shell')} class="btn btn-secondary text-sm px-3 py-2 shrink-0">
							<Copy class="w-3.5 h-3.5 mr-1" />
							{copiedField === 'shell' ? 'Copied!' : 'Copy'}
						</button>
					</div>
					<div class="flex items-center gap-2">
						<div class="flex-1 bg-gray-900 rounded-lg px-3 py-2 font-mono text-sm text-gray-200 truncate">
							<Terminal class="w-3.5 h-3.5 inline mr-1.5 text-blue-400" />{claudeCmd}
						</div>
						<button onclick={() => copyToClipboard(claudeCmd, 'claude')} class="btn btn-secondary text-sm px-3 py-2 shrink-0">
							<Copy class="w-3.5 h-3.5 mr-1" />
							{copiedField === 'claude' ? 'Copied!' : 'Copy'}
						</button>
					</div>
					<p class="text-xs text-gray-400 dark:text-gray-500">Paste into Warp or any terminal. For Claude, set ANTHROPIC_API_KEY in the container's env vars.</p>
				</div>
			</div>
		{/if}
	</div>
{:else if activeTab === 'logs'}
	<div id="panel-logs" role="tabpanel">
		<ContainerLogs containerId={container.id} />
	</div>
{:else if activeTab === 'stats'}
	<div id="panel-stats" role="tabpanel">
		<ContainerStats containerId={container.id} />
	</div>
{:else if activeTab === 'console'}
	<div id="panel-console" role="tabpanel">
		{#if isRunning}
			<ContainerTerminal containerId={container.id} />
		{:else}
			<div class="text-center py-8">
				<p class="text-gray-500 dark:text-gray-400">Container must be running to use the console</p>
				<button onclick={handleStart} class="btn btn-primary text-sm mt-3">
					<Play class="w-4 h-4 mr-1" />
					Start Container
				</button>
			</div>
		{/if}
	</div>
{:else if activeTab === 'files'}
	<div id="panel-files" role="tabpanel">
		{#if isRunning}
			<ContainerFiles containerId={container.id} workingDir={container.workingDir} />
		{:else}
			<div class="text-center py-8">
				<p class="text-gray-500 dark:text-gray-400">Container must be running to browse files</p>
				<button onclick={handleStart} class="btn btn-primary text-sm mt-3">
					<Play class="w-4 h-4 mr-1" />
					Start Container
				</button>
			</div>
		{/if}
	</div>
{/if}
