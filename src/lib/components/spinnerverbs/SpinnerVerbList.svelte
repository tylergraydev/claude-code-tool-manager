<script lang="ts">
	import { GripVertical, Pencil, Trash2 } from 'lucide-svelte';
	import { spinnerVerbLibrary } from '$lib/stores';
	import type { SpinnerVerb } from '$lib/types';

	type Props = {
		onEdit: (verb: SpinnerVerb) => void;
		onDelete: (verb: SpinnerVerb) => void;
	};

	let { onEdit, onDelete }: Props = $props();

	let draggedIndex = $state<number | null>(null);
	let dragOverIndex = $state<number | null>(null);

	function handleDragStart(index: number) {
		draggedIndex = index;
	}

	function handleDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		dragOverIndex = index;
	}

	function handleDragEnd() {
		if (draggedIndex !== null && dragOverIndex !== null && draggedIndex !== dragOverIndex) {
			const verbs = [...spinnerVerbLibrary.verbs];
			const [moved] = verbs.splice(draggedIndex, 1);
			verbs.splice(dragOverIndex, 0, moved);
			const ids = verbs.map((v) => v.id);
			spinnerVerbLibrary.reorder(ids);
		}
		draggedIndex = null;
		dragOverIndex = null;
	}

	async function handleToggle(verb: SpinnerVerb) {
		await spinnerVerbLibrary.update(verb.id, verb.verb, !verb.isEnabled);
	}
</script>

{#if spinnerVerbLibrary.verbs.length === 0}
	<div class="text-center py-12">
		<p class="text-gray-500 dark:text-gray-400">No spinner verbs yet. Add one to get started.</p>
	</div>
{:else}
	<div class="space-y-2">
		{#each spinnerVerbLibrary.verbs as verb, index (verb.id)}
			<div
				class="flex items-center gap-3 px-4 py-3 rounded-lg border transition-colors
					{dragOverIndex === index ? 'border-primary-400 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800'}
					{!verb.isEnabled ? 'opacity-50' : ''}"
				draggable="true"
				ondragstart={() => handleDragStart(index)}
				ondragover={(e) => handleDragOver(e, index)}
				ondragend={handleDragEnd}
				role="listitem"
			>
				<button
					class="cursor-grab text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
					aria-label="Drag to reorder"
				>
					<GripVertical class="w-4 h-4" />
				</button>

				<label class="flex items-center gap-3 flex-1 cursor-pointer">
					<input
						type="checkbox"
						checked={verb.isEnabled}
						onchange={() => handleToggle(verb)}
						class="w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-primary-600 focus:ring-primary-500"
					/>
					<span class="text-sm font-medium text-gray-900 dark:text-white">
						{verb.verb}
					</span>
				</label>

				<div class="flex items-center gap-1">
					<button
						onclick={() => onEdit(verb)}
						class="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						aria-label="Edit verb"
					>
						<Pencil class="w-4 h-4" />
					</button>
					<button
						onclick={() => onDelete(verb)}
						class="p-1.5 rounded-lg text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
						aria-label="Delete verb"
					>
						<Trash2 class="w-4 h-4" />
					</button>
				</div>
			</div>
		{/each}
	</div>
{/if}
