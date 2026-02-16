<script lang="ts">
	import { Plus, X, FolderOpen } from 'lucide-svelte';

	type Props = {
		directories: string[];
		onchange: (dirs: string[]) => void;
	};

	let { directories, onchange }: Props = $props();

	let newDir = $state('');
	let isAdding = $state(false);

	function addDirectory() {
		const dir = newDir.trim();
		if (!dir) return;
		if (directories.includes(dir)) return;
		onchange([...directories, dir]);
		newDir = '';
		isAdding = false;
	}

	function removeDirectory(index: number) {
		const updated = [...directories];
		updated.splice(index, 1);
		onchange(updated);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			addDirectory();
		} else if (e.key === 'Escape') {
			isAdding = false;
			newDir = '';
		}
	}
</script>

<div>
	<div class="flex items-center gap-2 mb-2">
		<FolderOpen class="w-4 h-4 text-gray-500 dark:text-gray-400" />
		<label class="text-sm font-medium text-gray-700 dark:text-gray-300">
			Additional Directories
		</label>
		{#if !isAdding}
			<button
				onclick={() => (isAdding = true)}
				class="flex items-center gap-1 px-2 py-0.5 text-xs rounded text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30"
			>
				<Plus class="w-3 h-3" />
				Add
			</button>
		{/if}
	</div>

	{#if directories.length > 0}
		<div class="space-y-1 mb-2">
			{#each directories as dir, index}
				<div
					class="flex items-center gap-2 px-3 py-1.5 bg-gray-50 dark:bg-gray-700/50 rounded-md group"
				>
					<code class="flex-1 text-sm text-gray-700 dark:text-gray-300">{dir}</code>
					<button
						onclick={() => removeDirectory(index)}
						class="p-0.5 text-gray-400 hover:text-red-500 dark:hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
					>
						<X class="w-3.5 h-3.5" />
					</button>
				</div>
			{/each}
		</div>
	{:else if !isAdding}
		<p class="text-xs text-gray-400 dark:text-gray-500 mb-2">
			No additional directories configured
		</p>
	{/if}

	{#if isAdding}
		<div class="flex items-center gap-2">
			<input
				type="text"
				bind:value={newDir}
				onkeydown={handleKeydown}
				placeholder="/path/to/directory"
				class="input text-sm flex-1 font-mono"
			/>
			<button onclick={addDirectory} class="btn btn-primary btn-sm">Add</button>
			<button
				onclick={() => {
					isAdding = false;
					newDir = '';
				}}
				class="btn btn-secondary btn-sm"
			>
				Cancel
			</button>
		</div>
	{/if}
</div>
