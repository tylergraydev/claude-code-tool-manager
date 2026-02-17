<script lang="ts">
	import { KNOWN_ENV_VARS, ENV_VAR_CATEGORIES } from '$lib/types';
	import { Plus, Search } from 'lucide-svelte';

	type Props = {
		existingKeys: string[];
		onselect: (key: string) => void;
	};

	let { existingKeys, onselect }: Props = $props();

	let selectedCategory = $state('');
	let searchQuery = $state('');

	const filteredVars = $derived(() => {
		return KNOWN_ENV_VARS.filter((v) => {
			if (selectedCategory && v.category !== selectedCategory) return false;
			if (searchQuery) {
				const q = searchQuery.toLowerCase();
				return v.key.toLowerCase().includes(q) || v.description.toLowerCase().includes(q);
			}
			return true;
		});
	});
</script>

<div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
	<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Known Environment Variables</h4>

	<div class="flex flex-wrap gap-2 mb-3">
		<div class="relative flex-1 min-w-[200px]">
			<Search class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
			<input
				type="text"
				bind:value={searchQuery}
				placeholder="Search variables..."
				class="input text-sm pl-8 w-full"
			/>
		</div>
		<select
			bind:value={selectedCategory}
			class="input text-sm"
		>
			<option value="">All categories</option>
			{#each ENV_VAR_CATEGORIES as category}
				<option value={category}>{category}</option>
			{/each}
		</select>
	</div>

	<div class="max-h-60 overflow-y-auto space-y-1">
		{#each filteredVars() as envVar}
			{@const isExisting = existingKeys.includes(envVar.key)}
			<div class="flex items-center justify-between px-2 py-1.5 rounded hover:bg-gray-100 dark:hover:bg-gray-800">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<code class="text-xs font-medium text-gray-900 dark:text-gray-100">{envVar.key}</code>
						<span class="text-[10px] px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400">
							{envVar.category}
						</span>
					</div>
					<p class="text-xs text-gray-500 dark:text-gray-400 truncate">{envVar.description}</p>
				</div>
				<button
					onclick={() => onselect(envVar.key)}
					disabled={isExisting}
					class="btn btn-ghost text-xs ml-2 shrink-0"
					class:opacity-50={isExisting}
					title={isExisting ? 'Already added' : `Add ${envVar.key}`}
				>
					<Plus class="w-3.5 h-3.5" />
				</button>
			</div>
		{/each}
		{#if filteredVars().length === 0}
			<p class="text-xs text-gray-400 dark:text-gray-500 italic text-center py-4">No matching variables found</p>
		{/if}
	</div>
</div>
