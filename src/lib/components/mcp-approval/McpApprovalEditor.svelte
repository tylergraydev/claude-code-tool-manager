<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save, Plus, X } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	let enableAll = $state<boolean | undefined>(settings.enableAllProjectMcpServers);
	let enabledServers = $state<string[]>([...(settings.enabledMcpjsonServers ?? [])]);
	let disabledServers = $state<string[]>([...(settings.disabledMcpjsonServers ?? [])]);
	let newEnabled = $state('');
	let newDisabled = $state('');

	$effect(() => {
		enableAll = settings.enableAllProjectMcpServers;
		enabledServers = [...(settings.enabledMcpjsonServers ?? [])];
		disabledServers = [...(settings.disabledMcpjsonServers ?? [])];
	});

	function handleToggleEnableAll() {
		enableAll = enableAll === undefined ? true : enableAll ? false : undefined;
	}

	function getTriStateLabel(value: boolean | undefined): string {
		if (value === undefined) return 'Not set';
		return value ? 'Enabled' : 'Disabled';
	}

	function getTriStateColor(value: boolean | undefined): string {
		if (value === undefined) return 'bg-gray-200 dark:bg-gray-600';
		return value ? 'bg-primary-600' : 'bg-red-400 dark:bg-red-600';
	}

	function getTriStatePosition(value: boolean | undefined): string {
		if (value === undefined) return 'translate-x-1';
		return value ? 'translate-x-6' : 'translate-x-3.5';
	}

	function addEnabled() {
		const val = newEnabled.trim();
		if (val && !enabledServers.includes(val)) {
			enabledServers = [...enabledServers, val];
			newEnabled = '';
		}
	}

	function removeEnabled(index: number) {
		enabledServers = enabledServers.filter((_, i) => i !== index);
	}

	function addDisabled() {
		const val = newDisabled.trim();
		if (val && !disabledServers.includes(val)) {
			disabledServers = [...disabledServers, val];
			newDisabled = '';
		}
	}

	function removeDisabled(index: number) {
		disabledServers = disabledServers.filter((_, i) => i !== index);
	}

	function handleEnabledKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addEnabled();
		}
	}

	function handleDisabledKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addDisabled();
		}
	}

	function handleSave() {
		onsave({
			...settings,
			enableAllProjectMcpServers: enableAll,
			enabledMcpjsonServers: enabledServers.length > 0 ? enabledServers : undefined,
			disabledMcpjsonServers: disabledServers.length > 0 ? disabledServers : undefined
		});
	}
</script>

<div class="space-y-6">
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">MCP Server Approval</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Control which project-level MCP servers are automatically approved
		</p>

		<div class="space-y-6">
			<!-- Tri-state toggle -->
			<div class="flex items-center justify-between">
				<div>
					<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
						Enable All Project MCP Servers
					</label>
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Automatically approve all MCP servers defined in project settings
					</p>
				</div>
				<div class="flex items-center gap-2">
					<span class="text-xs text-gray-500 dark:text-gray-400 min-w-[60px] text-right">
						{getTriStateLabel(enableAll)}
					</span>
					<button
						onclick={handleToggleEnableAll}
						class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {getTriStateColor(enableAll)}"
						title="Click to cycle: Not set → Enabled → Disabled → Not set"
					>
						<span
							class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {getTriStatePosition(enableAll)}"
						></span>
					</button>
				</div>
			</div>

			<!-- Enabled servers list -->
			<div>
				<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Enabled MCP Servers
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Specific MCP servers to enable from .mcp.json
				</p>
				<div class="flex gap-2 mb-2">
					<input
						type="text"
						bind:value={newEnabled}
						onkeydown={handleEnabledKeydown}
						placeholder="server-name"
						class="input flex-1"
					/>
					<button onclick={addEnabled} class="btn btn-ghost" title="Add server">
						<Plus class="w-4 h-4" />
					</button>
				</div>
				{#if enabledServers.length > 0}
					<div class="flex flex-wrap gap-2">
						{#each enabledServers as server, i}
							<span class="inline-flex items-center gap-1 px-2 py-1 rounded-md text-sm bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400 border border-green-200 dark:border-green-800">
								{server}
								<button
									onclick={() => removeEnabled(i)}
									class="hover:text-red-500 transition-colors"
									title="Remove"
								>
									<X class="w-3 h-3" />
								</button>
							</span>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Disabled servers list -->
			<div>
				<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Disabled MCP Servers
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Specific MCP servers to disable from .mcp.json
				</p>
				<div class="flex gap-2 mb-2">
					<input
						type="text"
						bind:value={newDisabled}
						onkeydown={handleDisabledKeydown}
						placeholder="server-name"
						class="input flex-1"
					/>
					<button onclick={addDisabled} class="btn btn-ghost" title="Add server">
						<Plus class="w-4 h-4" />
					</button>
				</div>
				{#if disabledServers.length > 0}
					<div class="flex flex-wrap gap-2">
						{#each disabledServers as server, i}
							<span class="inline-flex items-center gap-1 px-2 py-1 rounded-md text-sm bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400 border border-red-200 dark:border-red-800">
								{server}
								<button
									onclick={() => removeDisabled(i)}
									class="hover:text-red-500 transition-colors"
									title="Remove"
								>
									<X class="w-3 h-3" />
								</button>
							</span>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save MCP Approval Settings
		</button>
	</div>
</div>
