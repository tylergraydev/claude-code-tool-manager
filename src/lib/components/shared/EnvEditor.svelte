<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';

	type Props = {
		values: Record<string, string>;
		keyPlaceholder?: string;
		valuePlaceholder?: string;
		readonly?: boolean;
	};

	let {
		values = $bindable({}),
		keyPlaceholder = 'Variable name',
		valuePlaceholder = 'Value',
		readonly = false
	}: Props = $props();

	let entries = $state<[string, string][]>(Object.entries(values));

	$effect(() => {
		// Sync from props
		entries = Object.entries(values);
	});

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
		<div class="flex gap-2">
			<input
				type="text"
				{value}
				onchange={(e) => handleKeyChange(index, e.currentTarget.value)}
				placeholder={keyPlaceholder}
				class="input flex-1"
				disabled={readonly}
			/>
			<input
				type="text"
				value={value}
				onchange={(e) => handleValueChange(index, e.currentTarget.value)}
				placeholder={valuePlaceholder}
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
			Add variable
		</button>
	{/if}
</div>
