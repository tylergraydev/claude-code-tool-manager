<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { Save } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	let command = $state(settings.fileSuggestionCommand ?? '');

	$effect(() => {
		command = settings.fileSuggestionCommand ?? '';
	});

	function handleSave() {
		const hasValue = command.trim() !== '';
		onsave({
			...settings,
			fileSuggestionType: hasValue ? 'command' : undefined,
			fileSuggestionCommand: hasValue ? command.trim() : undefined
		});
	}
</script>

<div class="space-y-6">
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">File Suggestion</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Configure a custom script for <code class="text-xs">@</code> file autocomplete suggestions. The script receives a JSON object on stdin and should output file paths to stdout.
		</p>

		<div class="space-y-4">
			<div>
				<div class="flex items-center gap-2 mb-1">
					<label for="fs-command" class="text-sm font-medium text-gray-700 dark:text-gray-300">
						Command
					</label>
					<span class="px-1.5 py-0.5 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400">
						type: command
					</span>
				</div>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Path to a script or executable that provides file suggestions
				</p>
				<input
					id="fs-command"
					type="text"
					bind:value={command}
					placeholder="/path/to/suggest-files.sh"
					class="input w-full"
				/>
			</div>
		</div>
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save File Suggestion Settings
		</button>
	</div>
</div>
