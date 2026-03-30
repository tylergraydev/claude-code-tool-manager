<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { CLAUDE_MODELS } from '$lib/types';
	import { Save, Plus, X } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	type Override = { key: string; value: string };

	function recordToArray(record: Record<string, string> | undefined): Override[] {
		if (!record || Object.keys(record).length === 0) return [];
		return Object.entries(record).map(([key, value]) => ({ key, value }));
	}

	let overrides = $state<Override[]>(recordToArray(settings.modelOverrides));

	$effect(() => {
		overrides = recordToArray(settings.modelOverrides);
	});

	function addOverride() {
		overrides = [...overrides, { key: '', value: '' }];
	}

	function removeOverride(index: number) {
		overrides = overrides.filter((_, i) => i !== index);
	}

	function handleSave() {
		const validOverrides = overrides.filter((o) => o.key.trim() && o.value.trim());
		const record: Record<string, string> = {};
		for (const o of validOverrides) {
			record[o.key.trim()] = o.value.trim();
		}
		onsave({
			...settings,
			modelOverrides: Object.keys(record).length > 0 ? record : undefined
		});
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
	<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Model Overrides</h3>
	<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
		Map Anthropic model IDs to provider-specific IDs for Bedrock, Vertex, or Foundry deployments
	</p>

	<div class="space-y-3">
		{#if overrides.length > 0}
			<div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-center text-xs font-medium text-gray-500 dark:text-gray-400 px-1">
				<span>Anthropic Model ID</span>
				<span>Provider Model ID</span>
				<span class="w-8"></span>
			</div>
		{/if}

		{#each overrides as override, index}
			<div class="grid grid-cols-[1fr_1fr_auto] gap-2 items-center">
				<div class="relative">
					<input
						type="text"
						list="model-ids-{index}"
						bind:value={override.key}
						placeholder="e.g., claude-sonnet-4-5-20250929"
						class="input w-full text-sm"
					/>
					<datalist id="model-ids-{index}">
						{#each CLAUDE_MODELS as m}
							<option value={m.value}>{m.label}</option>
						{/each}
					</datalist>
				</div>
				<input
					type="text"
					bind:value={override.value}
					placeholder="e.g., us.anthropic.claude-sonnet-v2@bedrock"
					class="input w-full text-sm"
				/>
				<button
					onclick={() => removeOverride(index)}
					class="p-1.5 text-gray-400 hover:text-red-500 dark:hover:text-red-400 transition-colors"
					title="Remove override"
				>
					<X class="w-4 h-4" />
				</button>
			</div>
		{/each}

		<button
			onclick={addOverride}
			class="flex items-center gap-1.5 text-sm text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 transition-colors"
		>
			<Plus class="w-4 h-4" />
			Add Override
		</button>
	</div>

	<div class="mt-5 flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Model Overrides
		</button>
	</div>
</div>
