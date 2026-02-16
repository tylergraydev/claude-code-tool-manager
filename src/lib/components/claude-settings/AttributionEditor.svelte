<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save, X } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	const DEFAULT_COMMIT = 'Co-Authored-By: Claude <noreply@anthropic.com>';
	const DEFAULT_PR = 'Generated with [Claude Code](https://claude.ai/code)';

	let commitText = $state(settings.attributionCommit ?? '');
	let prText = $state(settings.attributionPr ?? '');
	let commitHasValue = $state(settings.attributionCommit !== undefined);
	let prHasValue = $state(settings.attributionPr !== undefined);

	// Reset local state when settings prop changes
	$effect(() => {
		commitText = settings.attributionCommit ?? '';
		prText = settings.attributionPr ?? '';
		commitHasValue = settings.attributionCommit !== undefined;
		prHasValue = settings.attributionPr !== undefined;
	});

	function handleSave() {
		onsave({
			scope: settings.scope,
			model: settings.model,
			availableModels: settings.availableModels,
			outputStyle: settings.outputStyle,
			language: settings.language,
			alwaysThinkingEnabled: settings.alwaysThinkingEnabled,
			attributionCommit: commitHasValue ? commitText : undefined,
			attributionPr: prHasValue ? prText : undefined
		});
	}

	function clearCommit() {
		commitHasValue = true;
		commitText = '';
	}

	function clearPr() {
		prHasValue = true;
		prText = '';
	}

	function unsetCommit() {
		commitHasValue = false;
		commitText = '';
	}

	function unsetPr() {
		prHasValue = false;
		prText = '';
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
	<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Attribution</h3>
	<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
		Customize text appended to git commits and PR descriptions. Set to empty to hide attribution.
	</p>

	<div class="space-y-4">
		<!-- Commit Attribution -->
		<div>
			<div class="flex items-center justify-between mb-1">
				<label
					for="commit-attribution"
					class="text-sm font-medium text-gray-700 dark:text-gray-300"
				>
					Commit Attribution
				</label>
				<div class="flex gap-1">
					{#if commitHasValue}
						<button
							onclick={unsetCommit}
							class="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
							title="Unset (use default)"
						>
							<X class="w-3.5 h-3.5" />
						</button>
					{:else}
						<button
							onclick={clearCommit}
							class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400"
						>
							Set to empty (hide)
						</button>
					{/if}
				</div>
			</div>
			{#if commitHasValue}
				<textarea
					id="commit-attribution"
					bind:value={commitText}
					placeholder="e.g. Co-Authored-By: Claude <noreply@anthropic.com>"
					rows={2}
					class="input text-sm w-full resize-y"
				></textarea>
				{#if commitText === ''}
					<p class="text-xs text-amber-600 dark:text-amber-400 mt-1">
						Empty string — commit attribution will be hidden
					</p>
				{/if}
			{:else}
				<p class="text-xs text-gray-500 dark:text-gray-400 italic">
					Not set — using default: "{DEFAULT_COMMIT}"
				</p>
			{/if}
		</div>

		<!-- PR Attribution -->
		<div>
			<div class="flex items-center justify-between mb-1">
				<label
					for="pr-attribution"
					class="text-sm font-medium text-gray-700 dark:text-gray-300"
				>
					PR Description Attribution
				</label>
				<div class="flex gap-1">
					{#if prHasValue}
						<button
							onclick={unsetPr}
							class="text-xs text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
							title="Unset (use default)"
						>
							<X class="w-3.5 h-3.5" />
						</button>
					{:else}
						<button
							onclick={clearPr}
							class="text-xs text-primary-600 hover:text-primary-700 dark:text-primary-400"
						>
							Set to empty (hide)
						</button>
					{/if}
				</div>
			</div>
			{#if prHasValue}
				<textarea
					id="pr-attribution"
					bind:value={prText}
					placeholder="e.g. Generated with Claude Code"
					rows={2}
					class="input text-sm w-full resize-y"
				></textarea>
				{#if prText === ''}
					<p class="text-xs text-amber-600 dark:text-amber-400 mt-1">
						Empty string — PR attribution will be hidden
					</p>
				{/if}
			{:else}
				<p class="text-xs text-gray-500 dark:text-gray-400 italic">
					Not set — using default: "{DEFAULT_PR}"
				</p>
			{/if}
		</div>

		<!-- Preview -->
		{#if commitHasValue || prHasValue}
			<div class="border-t border-gray-200 dark:border-gray-700 pt-4">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Preview</h4>
				<div class="space-y-2">
					{#if commitHasValue}
						<div class="bg-gray-50 dark:bg-gray-900/50 rounded p-3">
							<p class="text-xs text-gray-500 dark:text-gray-400 mb-1">Commit message footer:</p>
							<pre class="text-xs text-gray-700 dark:text-gray-300 whitespace-pre-wrap">{commitText || '(hidden — empty string)'}</pre>
						</div>
					{/if}
					{#if prHasValue}
						<div class="bg-gray-50 dark:bg-gray-900/50 rounded p-3">
							<p class="text-xs text-gray-500 dark:text-gray-400 mb-1">PR description footer:</p>
							<pre class="text-xs text-gray-700 dark:text-gray-300 whitespace-pre-wrap">{prText || '(hidden — empty string)'}</pre>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>

	<div class="mt-5 flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Attribution
		</button>
	</div>
</div>
