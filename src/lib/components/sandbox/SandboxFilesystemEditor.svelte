<script lang="ts">
	import type { SandboxFilesystemSettings } from '$lib/types';
	import { Plus, X } from 'lucide-svelte';

	type Props = {
		filesystem: SandboxFilesystemSettings;
		onchange: (filesystem: SandboxFilesystemSettings) => void;
	};

	let { filesystem, onchange }: Props = $props();

	let allowRead = $state<string[]>([...(filesystem.allowRead ?? [])]);
	let denyRead = $state<string[]>([...(filesystem.denyRead ?? [])]);
	let allowUnixSockets = $state<string[]>([...(filesystem.allowUnixSockets ?? [])]);

	let newAllowRead = $state('');
	let newDenyRead = $state('');
	let newUnixSocket = $state('');

	$effect(() => {
		allowRead = [...(filesystem.allowRead ?? [])];
		denyRead = [...(filesystem.denyRead ?? [])];
		allowUnixSockets = [...(filesystem.allowUnixSockets ?? [])];
	});

	function emitChange() {
		onchange({
			allowRead: allowRead.length > 0 ? allowRead : undefined,
			denyRead: denyRead.length > 0 ? denyRead : undefined,
			allowUnixSockets: allowUnixSockets.length > 0 ? allowUnixSockets : undefined
		});
	}

	function addAllowRead() {
		const trimmed = newAllowRead.trim();
		if (trimmed && !allowRead.includes(trimmed)) {
			allowRead = [...allowRead, trimmed];
			newAllowRead = '';
			emitChange();
		}
	}

	function removeAllowRead(index: number) {
		allowRead = allowRead.filter((_, i) => i !== index);
		emitChange();
	}

	function addDenyRead() {
		const trimmed = newDenyRead.trim();
		if (trimmed && !denyRead.includes(trimmed)) {
			denyRead = [...denyRead, trimmed];
			newDenyRead = '';
			emitChange();
		}
	}

	function removeDenyRead(index: number) {
		denyRead = denyRead.filter((_, i) => i !== index);
		emitChange();
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
</script>

<div class="space-y-4">
	<!-- Allow Read -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			Allow Read
		</label>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
			Re-allow reading paths within denied regions
		</p>
		<div class="flex gap-2 mb-2">
			<input
				type="text"
				bind:value={newAllowRead}
				placeholder="/opt/data/**"
				class="input text-sm flex-1"
				onkeydown={(e) => e.key === 'Enter' && addAllowRead()}
			/>
			<button
				onclick={addAllowRead}
				disabled={!newAllowRead.trim()}
				class="btn btn-ghost"
			>
				<Plus class="w-4 h-4" />
			</button>
		</div>
		{#if allowRead.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each allowRead as path, i}
					<span
						class="inline-flex items-center gap-1 px-2 py-1 rounded-md text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
					>
						<code>{path}</code>
						<button
							onclick={() => removeAllowRead(i)}
							class="text-gray-400 hover:text-red-500"
						>
							<X class="w-3 h-3" />
						</button>
					</span>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Deny Read -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			Deny Read
		</label>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
			Paths that subprocesses cannot read
		</p>
		<div class="flex gap-2 mb-2">
			<input
				type="text"
				bind:value={newDenyRead}
				placeholder="/etc/secrets/**"
				class="input text-sm flex-1"
				onkeydown={(e) => e.key === 'Enter' && addDenyRead()}
			/>
			<button
				onclick={addDenyRead}
				disabled={!newDenyRead.trim()}
				class="btn btn-ghost"
			>
				<Plus class="w-4 h-4" />
			</button>
		</div>
		{#if denyRead.length > 0}
			<div class="flex flex-wrap gap-2">
				{#each denyRead as path, i}
					<span
						class="inline-flex items-center gap-1 px-2 py-1 rounded-md text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
					>
						<code>{path}</code>
						<button
							onclick={() => removeDenyRead(i)}
							class="text-gray-400 hover:text-red-500"
						>
							<X class="w-3 h-3" />
						</button>
					</span>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Allow Unix Sockets -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
			Allow Unix Sockets
		</label>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
			Unix sockets to allow access to (e.g., for Docker)
		</p>
		<div class="flex gap-2 mb-2">
			<input
				type="text"
				bind:value={newUnixSocket}
				placeholder="/var/run/docker.sock"
				class="input text-sm flex-1"
				onkeydown={(e) => e.key === 'Enter' && addUnixSocket()}
			/>
			<button
				onclick={addUnixSocket}
				disabled={!newUnixSocket.trim()}
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
							class="text-gray-400 hover:text-red-500"
						>
							<X class="w-3 h-3" />
						</button>
					</span>
				{/each}
			</div>
		{/if}
	</div>
</div>
