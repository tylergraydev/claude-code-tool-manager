<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save, Plus, X, Trash2 } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	type PluginEntry = {
		name: string;
		mode: 'enabled' | 'disabled' | 'tools';
		tools: string[];
	};

	function parsePlugins(raw: Record<string, boolean | string[]> | undefined): PluginEntry[] {
		if (!raw) return [];
		return Object.entries(raw).map(([name, value]) => {
			if (value === true) return { name, mode: 'enabled' as const, tools: [] };
			if (value === false) return { name, mode: 'disabled' as const, tools: [] };
			return { name, mode: 'tools' as const, tools: [...value] };
		});
	}

	let plugins = $state<PluginEntry[]>(parsePlugins(settings.enabledPlugins));
	let newPluginName = $state('');

	$effect(() => {
		plugins = parsePlugins(settings.enabledPlugins);
		newPluginName = '';
	});

	function addPlugin() {
		const trimmed = newPluginName.trim();
		if (!trimmed || plugins.some((p) => p.name === trimmed)) return;
		plugins = [...plugins, { name: trimmed, mode: 'enabled', tools: [] }];
		newPluginName = '';
	}

	function removePlugin(index: number) {
		plugins = plugins.filter((_, i) => i !== index);
	}

	function setMode(index: number, mode: 'enabled' | 'disabled' | 'tools') {
		plugins[index] = { ...plugins[index], mode };
		if (mode !== 'tools') plugins[index].tools = [];
		plugins = [...plugins];
	}

	function updateTools(index: number, toolsStr: string) {
		plugins[index] = {
			...plugins[index],
			tools: toolsStr
				.split(',')
				.map((t) => t.trim())
				.filter(Boolean)
		};
		plugins = [...plugins];
	}

	function handleSave() {
		const enabledPlugins: Record<string, boolean | string[]> | undefined =
			plugins.length > 0
				? Object.fromEntries(
						plugins.map((p) => [
							p.name,
							p.mode === 'enabled' ? true : p.mode === 'disabled' ? false : p.tools
						])
					)
				: undefined;

		onsave({
			...settings,
			enabledPlugins
		});
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
	<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Enabled Plugins</h3>
	<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
		Configure which plugins are enabled and which tools they can access
	</p>

	{#if plugins.length > 0}
		<div class="space-y-3 mb-4">
			{#each plugins as plugin, i}
				<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg border border-gray-200 dark:border-gray-700 p-3">
					<div class="flex items-center gap-3">
						<code class="text-sm font-medium text-gray-900 dark:text-gray-100 flex-1 truncate">
							{plugin.name}
						</code>
						<select
							value={plugin.mode}
							onchange={(e) => setMode(i, (e.target as HTMLSelectElement).value as 'enabled' | 'disabled' | 'tools')}
							class="input text-xs w-36"
						>
							<option value="enabled">Enabled</option>
							<option value="disabled">Disabled</option>
							<option value="tools">Specific Tools</option>
						</select>
						<button
							onclick={() => removePlugin(i)}
							class="btn btn-ghost text-red-500 hover:text-red-700"
						>
							<Trash2 class="w-4 h-4" />
						</button>
					</div>
					{#if plugin.mode === 'tools'}
						<div class="mt-2">
							<input
								type="text"
								value={plugin.tools.join(', ')}
								oninput={(e) => updateTools(i, (e.target as HTMLInputElement).value)}
								placeholder="tool1, tool2, tool3"
								class="input text-xs w-full font-mono"
							/>
							<p class="text-[10px] text-gray-400 mt-0.5">Comma-separated list of allowed tool names</p>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{:else}
		<p class="text-xs text-gray-500 dark:text-gray-400 italic mb-4">
			No plugins configured
		</p>
	{/if}

	<div class="flex gap-2">
		<input
			type="text"
			bind:value={newPluginName}
			placeholder="Plugin name"
			class="input text-sm flex-1"
			onkeydown={(e) => e.key === 'Enter' && addPlugin()}
		/>
		<button
			onclick={addPlugin}
			disabled={!newPluginName.trim()}
			class="btn btn-ghost"
		>
			<Plus class="w-4 h-4" />
		</button>
	</div>

	<div class="flex justify-end mt-4">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Plugins
		</button>
	</div>
</div>
