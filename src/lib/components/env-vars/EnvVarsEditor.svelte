<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save, Plus, X, BookOpen } from 'lucide-svelte';
	import KnownEnvVarPicker from './KnownEnvVarPicker.svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	let entries = $state<[string, string][]>(Object.entries(settings.env ?? {}));
	let newKey = $state('');
	let newValue = $state('');
	let showPicker = $state(false);

	$effect(() => {
		entries = Object.entries(settings.env ?? {});
		newKey = '';
		newValue = '';
	});

	const KEY_PATTERN = /^[A-Za-z_][A-Za-z0-9_]*$/;

	function isValidKey(key: string): boolean {
		return KEY_PATTERN.test(key);
	}

	function addEntry() {
		const trimmedKey = newKey.trim();
		const trimmedValue = newValue.trim();
		if (!trimmedKey || !isValidKey(trimmedKey)) return;
		if (entries.some(([k]) => k === trimmedKey)) return;
		entries = [...entries, [trimmedKey, trimmedValue]];
		newKey = '';
		newValue = '';
	}

	function removeEntry(index: number) {
		entries = entries.filter((_, i) => i !== index);
	}

	function updateKey(index: number, key: string) {
		entries[index] = [key, entries[index][1]];
		entries = [...entries];
	}

	function updateValue(index: number, value: string) {
		entries[index] = [entries[index][0], value];
		entries = [...entries];
	}

	function handlePickerSelect(key: string) {
		newKey = key;
		showPicker = false;
	}

	function handleSave() {
		const env: Record<string, string> | undefined =
			entries.length > 0 ? Object.fromEntries(entries) : undefined;
		onsave({
			...settings,
			env
		});
	}

	const existingKeys = $derived(entries.map(([k]) => k));
</script>

<div class="space-y-6">
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<div class="flex items-center justify-between mb-1">
			<h3 class="text-base font-semibold text-gray-900 dark:text-white">Environment Variables</h3>
			<button
				onclick={() => (showPicker = !showPicker)}
				class="btn btn-ghost text-xs"
				title="Browse known Claude Code env vars"
			>
				<BookOpen class="w-4 h-4 mr-1" />
				{showPicker ? 'Hide' : 'Browse'} Known Vars
			</button>
		</div>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Set environment variables that Claude Code will use at runtime
		</p>

		{#if showPicker}
			<div class="mb-4">
				<KnownEnvVarPicker {existingKeys} onselect={handlePickerSelect} />
			</div>
		{/if}

		<!-- Existing entries -->
		{#if entries.length > 0}
			<div class="space-y-2 mb-4">
				{#each entries as [key, value], i}
					<div class="flex items-center gap-2">
						<input
							type="text"
							value={key}
							oninput={(e) => updateKey(i, (e.target as HTMLInputElement).value)}
							class="input text-sm font-mono w-48"
							class:border-red-400={!isValidKey(key)}
							placeholder="KEY"
						/>
						<span class="text-gray-400">=</span>
						<input
							type="text"
							value={value}
							oninput={(e) => updateValue(i, (e.target as HTMLInputElement).value)}
							class="input text-sm font-mono flex-1"
							placeholder="value"
						/>
						<button
							onclick={() => removeEntry(i)}
							class="btn btn-ghost text-red-500 hover:text-red-700"
						>
							<X class="w-4 h-4" />
						</button>
					</div>
				{/each}
			</div>
		{:else}
			<p class="text-xs text-gray-500 dark:text-gray-400 italic mb-4">
				No environment variables configured
			</p>
		{/if}

		<!-- Add new entry -->
		<div class="flex items-center gap-2">
			<input
				type="text"
				bind:value={newKey}
				placeholder="NEW_KEY"
				class="input text-sm font-mono w-48"
				class:border-red-400={newKey.trim() !== '' && !isValidKey(newKey.trim())}
				onkeydown={(e) => e.key === 'Enter' && addEntry()}
			/>
			<span class="text-gray-400">=</span>
			<input
				type="text"
				bind:value={newValue}
				placeholder="value"
				class="input text-sm font-mono flex-1"
				onkeydown={(e) => e.key === 'Enter' && addEntry()}
			/>
			<button
				onclick={addEntry}
				disabled={!newKey.trim() || !isValidKey(newKey.trim())}
				class="btn btn-ghost"
			>
				<Plus class="w-4 h-4" />
			</button>
		</div>
		{#if newKey.trim() !== '' && !isValidKey(newKey.trim())}
			<p class="text-xs text-red-500 mt-1">
				Key must start with a letter or underscore and contain only letters, digits, and underscores
			</p>
		{/if}
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Environment Variables
		</button>
	</div>
</div>
