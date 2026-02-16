<script lang="ts">
	import type { SandboxNetworkSettings } from '$lib/types';
	import { Plus, X } from 'lucide-svelte';

	type Props = {
		network: SandboxNetworkSettings;
		onchange: (network: SandboxNetworkSettings) => void;
	};

	let { network, onchange }: Props = $props();

	let allowAllUnixSockets = $state(network.allowAllUnixSockets ?? false);
	let allowUnixSockets = $state<string[]>([...(network.allowUnixSockets ?? [])]);
	let allowLocalBinding = $state(network.allowLocalBinding ?? false);
	let allowedDomains = $state<string[]>([...(network.allowedDomains ?? [])]);
	let httpProxyPort = $state<string>(network.httpProxyPort?.toString() ?? '');
	let socksProxyPort = $state<string>(network.socksProxyPort?.toString() ?? '');

	let newUnixSocket = $state('');
	let newDomain = $state('');

	// Reset local state when network prop changes
	$effect(() => {
		allowAllUnixSockets = network.allowAllUnixSockets ?? false;
		allowUnixSockets = [...(network.allowUnixSockets ?? [])];
		allowLocalBinding = network.allowLocalBinding ?? false;
		allowedDomains = [...(network.allowedDomains ?? [])];
		httpProxyPort = network.httpProxyPort?.toString() ?? '';
		socksProxyPort = network.socksProxyPort?.toString() ?? '';
	});

	function emitChange() {
		const httpPort = httpProxyPort ? parseInt(httpProxyPort) : undefined;
		const socksPort = socksProxyPort ? parseInt(socksProxyPort) : undefined;

		onchange({
			allowAllUnixSockets: allowAllUnixSockets || undefined,
			allowUnixSockets: allowUnixSockets.length > 0 ? allowUnixSockets : undefined,
			allowLocalBinding: allowLocalBinding || undefined,
			allowedDomains: allowedDomains.length > 0 ? allowedDomains : undefined,
			httpProxyPort: httpPort && !isNaN(httpPort) ? httpPort : undefined,
			socksProxyPort: socksPort && !isNaN(socksPort) ? socksPort : undefined
		});
	}

	function addUnixSocket() {
		const trimmed = newUnixSocket.trim();
		if (trimmed && !allowUnixSockets.includes(trimmed)) {
			allowUnixSockets = [...allowUnixSockets, trimmed];
			newUnixSocket = '';
			emitChange();
		}
	}

	function removeUnixSocket(index: number) {
		allowUnixSockets = allowUnixSockets.filter((_, i) => i !== index);
		emitChange();
	}

	function addDomain() {
		const trimmed = newDomain.trim();
		if (trimmed && !allowedDomains.includes(trimmed)) {
			allowedDomains = [...allowedDomains, trimmed];
			newDomain = '';
			emitChange();
		}
	}

	function removeDomain(index: number) {
		allowedDomains = allowedDomains.filter((_, i) => i !== index);
		emitChange();
	}

	function handleToggleAllUnixSockets() {
		allowAllUnixSockets = !allowAllUnixSockets;
		emitChange();
	}

	function handleToggleLocalBinding() {
		allowLocalBinding = !allowLocalBinding;
		emitChange();
	}

	function handlePortChange() {
		emitChange();
	}
</script>

<div class="space-y-4">
	<!-- Allow All Unix Sockets -->
	<div class="flex items-center justify-between">
		<div>
			<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
				Allow All Unix Sockets
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400">
				Allow connections to any Unix domain socket
			</p>
		</div>
		<button
			onclick={handleToggleAllUnixSockets}
			class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors
				{allowAllUnixSockets
				? 'bg-primary-600'
				: 'bg-gray-200 dark:bg-gray-600'}"
		>
			<span
				class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform
					{allowAllUnixSockets ? 'translate-x-6' : 'translate-x-1'}"
			></span>
		</button>
	</div>

	<!-- Allow Unix Sockets (specific paths) -->
	<div class:opacity-50={allowAllUnixSockets}>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			Allowed Unix Sockets
		</label>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
			Specific Unix socket paths to allow {allowAllUnixSockets ? '(disabled — all sockets allowed)' : ''}
		</p>
		<div class="flex gap-2 mb-2">
			<input
				type="text"
				bind:value={newUnixSocket}
				placeholder="/path/to/socket"
				disabled={allowAllUnixSockets}
				class="input text-sm flex-1"
				onkeydown={(e) => e.key === 'Enter' && addUnixSocket()}
			/>
			<button
				onclick={addUnixSocket}
				disabled={allowAllUnixSockets || !newUnixSocket.trim()}
				class="btn btn-ghost"
			>
				<Plus class="w-4 h-4" />
			</button>
		</div>
		{#if allowUnixSockets.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each allowUnixSockets as socket, i}
					<span
						class="inline-flex items-center gap-1 px-2 py-1 rounded-md text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
					>
						<code>{socket}</code>
						<button
							onclick={() => removeUnixSocket(i)}
							disabled={allowAllUnixSockets}
							class="text-gray-400 hover:text-red-500"
						>
							<X class="w-3 h-3" />
						</button>
					</span>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Allow Local Binding -->
	<div class="flex items-center justify-between">
		<div>
			<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
				Allow Local Binding
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400">
				Allow binding to local ports (macOS only)
			</p>
		</div>
		<button
			onclick={handleToggleLocalBinding}
			class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors
				{allowLocalBinding
				? 'bg-primary-600'
				: 'bg-gray-200 dark:bg-gray-600'}"
		>
			<span
				class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform
					{allowLocalBinding ? 'translate-x-6' : 'translate-x-1'}"
			></span>
		</button>
	</div>

	<!-- Allowed Domains -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			Allowed Domains
		</label>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
			Network domains the sandbox can access. Supports wildcards (e.g. <code>*.example.com</code>)
		</p>
		<div class="flex gap-2 mb-2">
			<input
				type="text"
				bind:value={newDomain}
				placeholder="*.example.com"
				class="input text-sm flex-1"
				onkeydown={(e) => e.key === 'Enter' && addDomain()}
			/>
			<button
				onclick={addDomain}
				disabled={!newDomain.trim()}
				class="btn btn-ghost"
			>
				<Plus class="w-4 h-4" />
			</button>
		</div>
		{#if allowedDomains.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each allowedDomains as domain, i}
					<span
						class="inline-flex items-center gap-1 px-2 py-1 rounded-md text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
					>
						<code>{domain}</code>
						<button
							onclick={() => removeDomain(i)}
							class="text-gray-400 hover:text-red-500"
						>
							<X class="w-3 h-3" />
						</button>
					</span>
				{/each}
			</div>
		{/if}
	</div>

	<!-- HTTP Proxy Port -->
	<div>
		<label for="http-proxy-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			HTTP Proxy Port
		</label>
		<input
			id="http-proxy-port"
			type="number"
			bind:value={httpProxyPort}
			onchange={handlePortChange}
			placeholder="e.g. 8080"
			min="1"
			max="65535"
			class="input text-sm w-full"
		/>
	</div>

	<!-- SOCKS Proxy Port -->
	<div>
		<label for="socks-proxy-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			SOCKS Proxy Port
		</label>
		<input
			id="socks-proxy-port"
			type="number"
			bind:value={socksProxyPort}
			onchange={handlePortChange}
			placeholder="e.g. 1080"
			min="1"
			max="65535"
			class="input text-sm w-full"
		/>
	</div>
</div>
