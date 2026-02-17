<script lang="ts">
	import type { MarketplaceDefinition, MarketplaceSource, MarketplaceSourceType } from '$lib/types';
	import { MARKETPLACE_SOURCE_TYPES } from '$lib/types';
	import { Save, X } from 'lucide-svelte';

	type Props = {
		name?: string;
		definition?: MarketplaceDefinition;
		onsave: (name: string, definition: MarketplaceDefinition) => void;
		oncancel: () => void;
	};

	let { name: initialName, definition, onsave, oncancel }: Props = $props();

	let name = $state(initialName ?? '');
	let sourceType = $state<MarketplaceSourceType>(definition?.source?.source ?? 'github');
	let installLocation = $state(definition?.installLocation ?? '');

	// Source-specific fields
	let repo = $state('');
	let gitUrl = $state('');
	let url = $state('');
	let npmPackage = $state('');
	let filePath = $state('');
	let dirPath = $state('');
	let hostPattern = $state('');
	let ref = $state('');
	let path = $state('');

	// Initialize from existing definition
	$effect(() => {
		if (definition?.source) {
			const src = definition.source;
			sourceType = src.source;
			if (src.source === 'github') {
				repo = src.repo;
				ref = src.ref ?? '';
				path = src.path ?? '';
			} else if (src.source === 'git') {
				gitUrl = src.url;
				ref = src.ref ?? '';
				path = src.path ?? '';
			} else if (src.source === 'url') {
				url = src.url;
			} else if (src.source === 'npm') {
				npmPackage = src.package;
			} else if (src.source === 'file') {
				filePath = src.path;
			} else if (src.source === 'directory') {
				dirPath = src.path;
			} else if (src.source === 'hostPattern') {
				hostPattern = src.hostPattern;
			}
		}
	});

	function buildSource(): MarketplaceSource {
		switch (sourceType) {
			case 'github':
				return { source: 'github', repo, ...(ref ? { ref } : {}), ...(path ? { path } : {}) };
			case 'git':
				return { source: 'git', url: gitUrl, ...(ref ? { ref } : {}), ...(path ? { path } : {}) };
			case 'url':
				return { source: 'url', url };
			case 'npm':
				return { source: 'npm', package: npmPackage };
			case 'file':
				return { source: 'file', path: filePath };
			case 'directory':
				return { source: 'directory', path: dirPath };
			case 'hostPattern':
				return { source: 'hostPattern', hostPattern };
		}
	}

	function handleSave() {
		if (!name.trim()) return;
		const def: MarketplaceDefinition = {
			source: buildSource(),
			...(installLocation ? { installLocation } : {})
		};
		onsave(name.trim(), def);
	}

	function isValid(): boolean {
		if (!name.trim()) return false;
		switch (sourceType) {
			case 'github': return !!repo.trim();
			case 'git': return !!gitUrl.trim();
			case 'url': return !!url.trim();
			case 'npm': return !!npmPackage.trim();
			case 'file': return !!filePath.trim();
			case 'directory': return !!dirPath.trim();
			case 'hostPattern': return !!hostPattern.trim();
		}
	}
</script>

<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg border border-gray-200 dark:border-gray-700 p-4 space-y-3">
	<div>
		<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Marketplace Name</label>
		<input
			type="text"
			bind:value={name}
			placeholder="my-marketplace"
			class="input text-sm w-full mt-1"
			disabled={!!initialName}
		/>
	</div>

	<div>
		<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Source Type</label>
		<select bind:value={sourceType} class="input text-sm w-full mt-1">
			{#each MARKETPLACE_SOURCE_TYPES as type}
				<option value={type.value}>{type.label}</option>
			{/each}
		</select>
	</div>

	{#if sourceType === 'github'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Repository <span class="text-red-500">*</span></label>
			<input type="text" bind:value={repo} placeholder="owner/repo" class="input text-sm w-full mt-1" />
		</div>
		<div class="grid grid-cols-2 gap-2">
			<div>
				<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Ref (branch/tag)</label>
				<input type="text" bind:value={ref} placeholder="main" class="input text-sm w-full mt-1" />
			</div>
			<div>
				<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Path</label>
				<input type="text" bind:value={path} placeholder="/" class="input text-sm w-full mt-1" />
			</div>
		</div>
	{:else if sourceType === 'git'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Git URL <span class="text-red-500">*</span></label>
			<input type="text" bind:value={gitUrl} placeholder="https://github.com/..." class="input text-sm w-full mt-1" />
		</div>
		<div class="grid grid-cols-2 gap-2">
			<div>
				<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Ref (branch/tag)</label>
				<input type="text" bind:value={ref} placeholder="main" class="input text-sm w-full mt-1" />
			</div>
			<div>
				<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Path</label>
				<input type="text" bind:value={path} placeholder="/" class="input text-sm w-full mt-1" />
			</div>
		</div>
	{:else if sourceType === 'url'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">URL <span class="text-red-500">*</span></label>
			<input type="text" bind:value={url} placeholder="https://..." class="input text-sm w-full mt-1" />
		</div>
	{:else if sourceType === 'npm'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Package <span class="text-red-500">*</span></label>
			<input type="text" bind:value={npmPackage} placeholder="@scope/package" class="input text-sm w-full mt-1" />
		</div>
	{:else if sourceType === 'file'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">File Path <span class="text-red-500">*</span></label>
			<input type="text" bind:value={filePath} placeholder="/path/to/file" class="input text-sm w-full mt-1" />
		</div>
	{:else if sourceType === 'directory'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Directory Path <span class="text-red-500">*</span></label>
			<input type="text" bind:value={dirPath} placeholder="/path/to/dir" class="input text-sm w-full mt-1" />
		</div>
	{:else if sourceType === 'hostPattern'}
		<div>
			<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Host Pattern <span class="text-red-500">*</span></label>
			<input type="text" bind:value={hostPattern} placeholder="*.example.com" class="input text-sm w-full mt-1" />
		</div>
	{/if}

	<div>
		<label class="text-xs font-medium text-gray-600 dark:text-gray-400">Install Location (optional)</label>
		<input type="text" bind:value={installLocation} placeholder="/path/to/install" class="input text-sm w-full mt-1" />
	</div>

	<div class="flex justify-end gap-2 pt-2">
		<button onclick={oncancel} class="btn btn-ghost text-sm">
			<X class="w-4 h-4 mr-1" />
			Cancel
		</button>
		<button onclick={handleSave} disabled={!isValid()} class="btn btn-primary text-sm">
			<Save class="w-4 h-4 mr-1" />
			{initialName ? 'Update' : 'Add'} Marketplace
		</button>
	</div>
</div>
