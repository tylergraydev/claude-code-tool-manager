<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import { i18n } from '$lib/i18n';

	type Props = {
		values: Record<string, string>;
		keyPlaceholder?: string;
		valuePlaceholder?: string;
		readonly?: boolean;
	};

	let {
		values = $bindable({}),
		keyPlaceholder = i18n.t('envEditor.variableName'),
		valuePlaceholder = i18n.t('envEditor.value'),
		readonly = false
	}: Props = $props();

	let entries = $state<[string, string][]>(Object.entries(values));

	$effect(() => {
		// Sync from props
		entries = Object.entries(values);
	});

	let duplicateKeys = $derived.by(() => {
		const seen = new Map<string, number[]>();
		entries.forEach(([key], i) => {
			const trimmed = key.trim();
			if (trimmed) {
				const indices = seen.get(trimmed) || [];
				indices.push(i);
				seen.set(trimmed, indices);
			}
		});
		const dupes = new Set<number>();
		for (const indices of seen.values()) {
			if (indices.length > 1) indices.forEach((i) => dupes.add(i));
		}
		return dupes;
	});

	function isValidEnvKey(key: string): boolean {
		return /^[a-zA-Z_][a-zA-Z0-9_]*$/.test(key);
	}

	function updateValues() {
		const newValues: Record<string, string> = {};
		for (const [key, value] of entries) {
			if (key.trim()) {
				newValues[key.trim()] = value;
			}
		}
		values = newValues;
	}

	function addEntry() {
		entries = [...entries, ['', '']];
	}

	function removeEntry(index: number) {
		entries = entries.filter((_, i) => i !== index);
		updateValues();
	}

	function handleKeyChange(index: number, newKey: string) {
		entries[index] = [newKey, entries[index][1]];
		updateValues();
	}

	function handleValueChange(index: number, newValue: string) {
		entries[index] = [entries[index][0], newValue];
		updateValues();
	}
</script>

<div class="space-y-2">
	{#each entries as [key, value], index}
		<div class="flex gap-2 items-start">
			<div class="flex-1">
				<input
					type="text"
					value={key}
					onchange={(e) => handleKeyChange(index, e.currentTarget.value)}
					placeholder={keyPlaceholder}
					aria-label="Variable name for entry {index + 1}"
					aria-invalid={key.trim() && !isValidEnvKey(key.trim()) ? 'true' : undefined}
					class="input w-full {duplicateKeys.has(index) ? 'border-yellow-500 dark:border-yellow-400' : ''} {key.trim() && !isValidEnvKey(key.trim()) ? 'border-red-500 dark:border-red-400' : ''}"
					disabled={readonly}
				/>
				{#if key.trim() && !isValidEnvKey(key.trim())}
					<p class="text-xs text-red-500 mt-0.5" role="alert">Letters, numbers, underscores only</p>
				{:else if duplicateKeys.has(index)}
					<p class="text-xs text-yellow-500 mt-0.5" role="alert">Duplicate key</p>
				{/if}
			</div>
			<input
				type="text"
				value={value}
				onchange={(e) => handleValueChange(index, e.currentTarget.value)}
				placeholder={valuePlaceholder}
				aria-label="Value for {key.trim() || `entry ${index + 1}`}"
				class="input flex-[2]"
				disabled={readonly}
			/>
			{#if !readonly}
				<button
					type="button"
					onclick={() => removeEntry(index)}
					class="btn btn-ghost text-red-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
				>
					<Trash2 class="w-4 h-4" />
				</button>
			{/if}
		</div>
	{/each}

	{#if !readonly}
		<button
			type="button"
			onclick={addEntry}
			class="btn btn-ghost text-gray-500 w-full justify-center border-2 border-dashed border-gray-200 dark:border-gray-700"
		>
			<Plus class="w-4 h-4 mr-2" />
			{i18n.t('envEditor.addVariable')}
		</button>
	{/if}
</div>
