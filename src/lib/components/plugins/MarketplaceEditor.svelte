<script lang="ts">
	import type { ClaudeSettings, MarketplaceDefinition } from '$lib/types';
	import { Save, Plus, Pencil, Trash2 } from 'lucide-svelte';
	import MarketplaceSourceForm from './MarketplaceSourceForm.svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	type MarketplaceEntry = {
		name: string;
		definition: MarketplaceDefinition;
	};

	function parseMarketplaces(raw: Record<string, MarketplaceDefinition> | undefined): MarketplaceEntry[] {
		if (!raw) return [];
		return Object.entries(raw).map(([name, definition]) => ({ name, definition }));
	}

	let marketplaces = $state<MarketplaceEntry[]>(parseMarketplaces(settings.extraKnownMarketplaces));
	let editingIndex = $state<number | null>(null);
	let addingNew = $state(false);

	$effect(() => {
		marketplaces = parseMarketplaces(settings.extraKnownMarketplaces);
		editingIndex = null;
		addingNew = false;
	});

	function getSourceSummary(def: MarketplaceDefinition): string {
		const src = def.source;
		switch (src.source) {
			case 'github': return src.repo;
			case 'git': return src.url;
			case 'url': return src.url;
			case 'npm': return src.package;
			case 'file': return src.path;
			case 'directory': return src.path;
			case 'hostPattern': return src.hostPattern;
		}
	}

	function handleAddSave(name: string, definition: MarketplaceDefinition) {
		marketplaces = [...marketplaces, { name, definition }];
		addingNew = false;
	}

	function handleEditSave(name: string, definition: MarketplaceDefinition) {
		if (editingIndex !== null) {
			marketplaces[editingIndex] = { name, definition };
			marketplaces = [...marketplaces];
			editingIndex = null;
		}
	}

	function removeMarketplace(index: number) {
		marketplaces = marketplaces.filter((_, i) => i !== index);
	}

	function handleSave() {
		const extraKnownMarketplaces: Record<string, MarketplaceDefinition> | undefined =
			marketplaces.length > 0
				? Object.fromEntries(marketplaces.map((m) => [m.name, m.definition]))
				: undefined;

		onsave({
			...settings,
			extraKnownMarketplaces
		});
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
	<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Extra Marketplaces</h3>
	<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
		Add custom marketplace sources for discovering and installing plugins
	</p>

	{#if marketplaces.length > 0}
		<div class="space-y-3 mb-4">
			{#each marketplaces as marketplace, i}
				{#if editingIndex === i}
					<MarketplaceSourceForm
						name={marketplace.name}
						definition={marketplace.definition}
						onsave={handleEditSave}
						oncancel={() => (editingIndex = null)}
					/>
				{:else}
					<div class="flex items-center gap-3 bg-gray-50 dark:bg-gray-900/50 rounded-lg border border-gray-200 dark:border-gray-700 px-3 py-2">
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-2">
								<code class="text-sm font-medium text-gray-900 dark:text-gray-100">{marketplace.name}</code>
								<span class="text-[10px] px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300">
									{marketplace.definition.source.source}
								</span>
							</div>
							<p class="text-xs text-gray-500 dark:text-gray-400 truncate">
								{getSourceSummary(marketplace.definition)}
							</p>
						</div>
						<button
							onclick={() => (editingIndex = i)}
							class="btn btn-ghost"
						>
							<Pencil class="w-3.5 h-3.5" />
						</button>
						<button
							onclick={() => removeMarketplace(i)}
							class="btn btn-ghost text-red-500 hover:text-red-700"
						>
							<Trash2 class="w-3.5 h-3.5" />
						</button>
					</div>
				{/if}
			{/each}
		</div>
	{:else if !addingNew}
		<p class="text-xs text-gray-500 dark:text-gray-400 italic mb-4">
			No extra marketplaces configured
		</p>
	{/if}

	{#if addingNew}
		<div class="mb-4">
			<MarketplaceSourceForm
				onsave={handleAddSave}
				oncancel={() => (addingNew = false)}
			/>
		</div>
	{:else}
		<button
			onclick={() => (addingNew = true)}
			class="btn btn-ghost text-sm mb-4"
		>
			<Plus class="w-4 h-4 mr-1" />
			Add Marketplace
		</button>
	{/if}

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Marketplaces
		</button>
	</div>
</div>
