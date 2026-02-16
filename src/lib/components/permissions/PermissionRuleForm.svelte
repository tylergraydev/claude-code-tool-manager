<script lang="ts">
	import type { PermissionCategory } from '$lib/types';
	import { PERMISSION_TOOL_NAMES } from '$lib/types';
	import { X, Code, Wand2 } from 'lucide-svelte';

	type Props = {
		category: PermissionCategory;
		onsubmit: (rule: string) => void;
		onclose: () => void;
	};

	let { category, onsubmit, onclose }: Props = $props();

	let selectedTool = $state('Bash');
	let specifier = $state('');
	let rawMode = $state(false);
	let rawRule = $state('');

	const currentToolHint = $derived(
		PERMISSION_TOOL_NAMES.find((t) => t.value === selectedTool)?.hint ?? ''
	);

	const preview = $derived.by(() => {
		if (rawMode) return rawRule;
		if (!specifier) return selectedTool;
		return `${selectedTool}(${specifier})`;
	});

	function handleSubmit() {
		const rule = rawMode ? rawRule.trim() : preview;
		if (!rule) return;
		onsubmit(rule);
	}

	const categoryLabels: Record<PermissionCategory, string> = {
		allow: 'Allow',
		deny: 'Deny',
		ask: 'Ask'
	};

	const categoryColors: Record<PermissionCategory, string> = {
		deny: 'bg-red-600 hover:bg-red-700',
		ask: 'bg-amber-600 hover:bg-amber-700',
		allow: 'bg-green-600 hover:bg-green-700'
	};
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onkeydown={(e) => e.key === 'Escape' && onclose()}
>
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-full max-w-lg mx-4" onclick={(e) => e.stopPropagation()}>
		<!-- Header -->
		<div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
			<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
				Add {categoryLabels[category]} Rule
			</h3>
			<button
				onclick={onclose}
				class="p-1 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
			>
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Body -->
		<div class="px-6 py-4 space-y-4">
			<!-- Mode toggle -->
			<div class="flex items-center gap-2">
				<button
					onclick={() => (rawMode = false)}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors
						{!rawMode ? 'bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
				>
					<Wand2 class="w-4 h-4" />
					Builder
				</button>
				<button
					onclick={() => (rawMode = true)}
					class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors
						{rawMode ? 'bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
				>
					<Code class="w-4 h-4" />
					Raw
				</button>
			</div>

			{#if rawMode}
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Permission Rule
					</label>
					<input
						type="text"
						bind:value={rawRule}
						placeholder="e.g. Bash(npm run *)"
						class="input w-full font-mono"
					/>
					<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
						Format: ToolName or ToolName(specifier pattern)
					</p>
				</div>
			{:else}
				<!-- Tool selector -->
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Tool
					</label>
					<select bind:value={selectedTool} class="input w-full">
						{#each PERMISSION_TOOL_NAMES as tool}
							<option value={tool.value}>{tool.label}</option>
						{/each}
					</select>
					<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">{currentToolHint}</p>
				</div>

				<!-- Specifier -->
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
						Specifier <span class="text-gray-400">(optional)</span>
					</label>
					<input
						type="text"
						bind:value={specifier}
						placeholder={selectedTool === 'Bash' ? 'npm run *' : selectedTool === 'Read' ? '.env*' : ''}
						class="input w-full font-mono"
					/>
					<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
						Use * for wildcard matching. Leave empty to match all uses of this tool.
					</p>
				</div>
			{/if}

			<!-- Preview -->
			<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
				<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Preview</p>
				<code class="text-sm font-mono text-gray-800 dark:text-gray-200">{preview || '...'}</code>
			</div>
		</div>

		<!-- Footer -->
		<div class="flex justify-end gap-3 px-6 py-4 border-t border-gray-200 dark:border-gray-700">
			<button onclick={onclose} class="btn btn-secondary">Cancel</button>
			<button
				onclick={handleSubmit}
				disabled={!preview}
				class="btn text-white {categoryColors[category]} disabled:opacity-50 disabled:cursor-not-allowed"
			>
				Add Rule
			</button>
		</div>
	</div>
</div>
