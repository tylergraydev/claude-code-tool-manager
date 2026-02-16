<script lang="ts">
	import type { SpinnerVerb } from '$lib/types';

	type Props = {
		initialValues?: SpinnerVerb | null;
		onSubmit: (verb: string) => void;
		onCancel: () => void;
	};

	let { initialValues = null, onSubmit, onCancel }: Props = $props();

	let verb = $state(initialValues?.verb ?? '');

	const isValid = $derived(verb.trim().length > 0);

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!isValid) return;
		onSubmit(verb.trim());
	}
</script>

<form onsubmit={handleSubmit} class="space-y-4">
	<div>
		<label
			for="spinner-verb"
			class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
		>
			Verb <span class="text-red-500">*</span>
		</label>
		<input
			id="spinner-verb"
			type="text"
			bind:value={verb}
			placeholder="e.g., Pondering, Crafting, Brewing"
			class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary-500 focus:border-transparent"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			The verb shown in Claude Code's spinner while it's working
		</p>
	</div>

	<div class="flex justify-end gap-3 pt-2">
		<button
			type="button"
			onclick={onCancel}
			class="px-4 py-2 text-sm font-medium rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
		>
			Cancel
		</button>
		<button
			type="submit"
			disabled={!isValid}
			class="px-4 py-2 text-sm font-medium rounded-lg bg-primary-600 text-white hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
		>
			{initialValues ? 'Update' : 'Add'} Verb
		</button>
	</div>
</form>
