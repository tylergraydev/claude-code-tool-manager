<script lang="ts">
	import { containerLibrary, notifications } from '$lib/stores';
	import type { Container } from '$lib/types';
	import ContainerStatusBadge from './ContainerStatus.svelte';
	import ContainerActions from './ContainerActions.svelte';
	import ContainerLogs from './ContainerLogs.svelte';
	import ContainerStats from './ContainerStats.svelte';
	import { X, Terminal } from 'lucide-svelte';

	let { container, onClose }: {
		container: Container;
		onClose: () => void;
	} = $props();

	let activeTab = $state<'overview' | 'logs' | 'stats' | 'exec'>('overview');
	let execCommand = $state('');
	let execOutput = $state('');
	let execRunning = $state(false);
	let actionInProgress = $state(false);

	const status = $derived(containerLibrary.getStatus(container.id));
	const dockerStatus = $derived(status?.dockerStatus || 'not_created');

	const tabs = ['overview', 'logs', 'stats', 'exec'] as const;

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	function handleTabKeydown(e: KeyboardEvent) {
		const currentIndex = tabs.indexOf(activeTab);
		let nextIndex = -1;
		if (e.key === 'ArrowRight') {
			e.preventDefault();
			nextIndex = (currentIndex + 1) % tabs.length;
		} else if (e.key === 'ArrowLeft') {
			e.preventDefault();
			nextIndex = (currentIndex - 1 + tabs.length) % tabs.length;
		} else if (e.key === 'Home') {
			e.preventDefault();
			nextIndex = 0;
		} else if (e.key === 'End') {
			e.preventDefault();
			nextIndex = tabs.length - 1;
		}
		if (nextIndex >= 0) {
			activeTab = tabs[nextIndex];
		}
	}

	async function handleExec() {
		if (!execCommand.trim() || execRunning) return;
		execRunning = true;
		const cmd = execCommand;
		execCommand = '';
		try {
			const result = await containerLibrary.exec(container.id, ['sh', '-c', cmd]);
			execOutput += `$ ${cmd}\n${result.stdout}${result.stderr}\n`;
		} catch (e) {
			execOutput += `$ ${cmd}\nError: ${e}\n`;
		} finally {
			execRunning = false;
		}
	}

	async function withAction(fn: () => Promise<void>) {
		if (actionInProgress) return;
		actionInProgress = true;
		try { await fn(); } finally { actionInProgress = false; }
	}

	async function handleStart() { await withAction(async () => { await containerLibrary.startContainer(container.id); notifications.success('Started'); }); }
	async function handleStop() { await withAction(async () => { await containerLibrary.stopContainer(container.id); notifications.success('Stopped'); }); }
	async function handleRestart() { await withAction(async () => { await containerLibrary.restartContainer(container.id); notifications.success('Restarted'); }); }
	async function handleRemove() { await withAction(async () => { await containerLibrary.removeContainer(container.id); notifications.success('Removed'); }); }
	async function handleBuild() { await withAction(async () => { await containerLibrary.buildImage(container.id); notifications.success('Built'); }); }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" role="dialog" aria-modal="true" onkeydown={handleKeydown}
	onclick={(e) => e.target === e.currentTarget && onClose()}>
	<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-4xl mx-4 max-h-[85vh] flex flex-col">
		<div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
			<div class="flex items-center gap-3">
				<span class="text-2xl">{container.icon || '📦'}</span>
				<div>
					<div class="flex items-center gap-2">
						<h2 class="text-lg font-semibold text-gray-900 dark:text-white">{container.name}</h2>
						<ContainerStatusBadge status={dockerStatus} />
					</div>
					{#if container.description}
						<p class="text-sm text-gray-500 dark:text-gray-400">{container.description}</p>
					{/if}
				</div>
			</div>
			<div class="flex items-center gap-2">
				<ContainerActions status={dockerStatus}
					disabled={actionInProgress}
					onBuild={container.dockerfile ? handleBuild : undefined}
					onStart={handleStart} onStop={handleStop} onRestart={handleRestart} onRemove={handleRemove} />
				<button onclick={onClose} class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 hover:bg-gray-100 dark:hover:text-gray-300 dark:hover:bg-gray-700 transition-colors" aria-label="Close">
					<X class="w-5 h-5" aria-hidden="true" />
				</button>
			</div>
		</div>

		<div class="flex border-b border-gray-200 dark:border-gray-700" role="tablist" aria-label="Container details">
			{#each tabs as tab}
				{@const isActive = activeTab === tab}
				<button
					role="tab"
					aria-selected={isActive}
					aria-controls="container-tabpanel"
					id="container-tab-{tab}"
					tabindex={isActive ? 0 : -1}
					onclick={() => activeTab = tab}
					onkeydown={handleTabKeydown}
					class="px-4 py-2 text-sm font-medium border-b-2 transition-colors
						{isActive ? 'border-primary-500 text-primary-600 dark:text-primary-400' : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'}">
					{tab.charAt(0).toUpperCase() + tab.slice(1)}
				</button>
			{/each}
		</div>

		<div id="container-tabpanel" role="tabpanel" aria-labelledby="container-tab-{activeTab}" class="flex-1 overflow-auto">
			{#if activeTab === 'overview'}
				<div class="p-4 space-y-3">
					<div class="grid grid-cols-2 gap-4 text-sm">
						<div><span class="text-gray-500 dark:text-gray-400">Type:</span> <span class="text-gray-900 dark:text-white">{container.containerType}</span></div>
						<div><span class="text-gray-500 dark:text-gray-400">Image:</span> <span class="text-gray-900 dark:text-white">{container.image || 'N/A'}</span></div>
						<div><span class="text-gray-500 dark:text-gray-400">Working Dir:</span> <span class="text-gray-900 dark:text-white">{container.workingDir || 'N/A'}</span></div>
						<div class="min-w-0"><span class="text-gray-500 dark:text-gray-400">Docker ID:</span> <span class="text-gray-900 dark:text-white font-mono text-xs break-all">{container.dockerContainerId || 'N/A'}</span></div>
					</div>
					{#if container.ports && container.ports.length > 0}
						<div>
							<p class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Ports</p>
							<div class="flex flex-wrap gap-2">
								{#each container.ports as port}
									<span class="text-xs px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded">{port.hostPort}:{port.containerPort}/{port.protocol || 'tcp'}</span>
								{/each}
							</div>
						</div>
					{/if}
					{#if container.env && Object.keys(container.env).length > 0}
						<div>
							<p class="text-sm font-medium text-gray-500 dark:text-gray-400 mb-1">Environment</p>
							<div class="font-mono text-xs space-y-0.5">
								{#each Object.entries(container.env) as [key, value]}
									<div><span class="text-blue-600 dark:text-blue-400">{key}</span>=<span class="text-gray-600 dark:text-gray-400">{value}</span></div>
								{/each}
							</div>
						</div>
					{/if}
				</div>
			{:else if activeTab === 'logs'}
				<div class="h-96">
					<ContainerLogs containerId={container.id} />
				</div>
			{:else if activeTab === 'stats'}
				<ContainerStats containerId={container.id} />
			{:else if activeTab === 'exec'}
				<div class="p-4 flex flex-col h-96">
					<div class="flex-1 bg-gray-950 rounded-lg p-3 font-mono text-xs text-gray-300 overflow-auto mb-2 whitespace-pre break-all">{#if execOutput}{execOutput}{:else}<span class="text-gray-600 italic">Run a command...</span>{/if}</div>
					<div class="flex gap-2">
						<input type="text" bind:value={execCommand} placeholder="Enter command..."
							disabled={execRunning || dockerStatus !== 'running'}
							onkeydown={(e) => e.key === 'Enter' && handleExec()}
							class="input flex-1 min-w-0 font-mono" />
						<button onclick={handleExec} disabled={execRunning || dockerStatus !== 'running' || !execCommand.trim()}
							class="btn btn-primary shrink-0">
							<Terminal class="w-4 h-4" aria-hidden="true" />
						</button>
					</div>
					{#if dockerStatus !== 'running'}
						<p class="text-xs text-gray-500 mt-1">Container must be running to execute commands</p>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>
