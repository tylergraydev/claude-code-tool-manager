<script lang="ts">
	import {
		CLAUDE_MODELS,
		AVAILABLE_MODEL_SHORTCUTS,
		OUTPUT_STYLES,
		COMMON_LANGUAGES
	} from '$lib/types';
	import type { ClaudeSettings } from '$lib/types';
	import { Save } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	let model = $state(settings.model ?? '');
	let availableModels = $state<string[]>([...settings.availableModels]);
	let outputStyle = $state(settings.outputStyle ?? '');
	let language = $state(settings.language ?? '');
	let alwaysThinkingEnabled = $state<boolean | undefined>(settings.alwaysThinkingEnabled);

	// Reset local state when settings prop changes
	$effect(() => {
		model = settings.model ?? '';
		availableModels = [...settings.availableModels];
		outputStyle = settings.outputStyle ?? '';
		language = settings.language ?? '';
		alwaysThinkingEnabled = settings.alwaysThinkingEnabled;
	});

	function handleSave() {
		onsave({
			scope: settings.scope,
			model: model || undefined,
			availableModels,
			outputStyle: outputStyle || undefined,
			language: language || undefined,
			alwaysThinkingEnabled
		});
	}

	function toggleAvailableModel(shortcut: string) {
		if (availableModels.includes(shortcut)) {
			availableModels = availableModels.filter((m) => m !== shortcut);
		} else {
			availableModels = [...availableModels, shortcut];
		}
	}

	function handleThinkingChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		const val = target.value;
		if (val === '') {
			alwaysThinkingEnabled = undefined;
		} else {
			alwaysThinkingEnabled = val === 'true';
		}
	}
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
	<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-4">Model & Output</h3>

	<div class="space-y-4">
		<!-- Model -->
		<div>
			<label
				for="model-select"
				class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
			>
				Default Model
			</label>
			<select
				id="model-select"
				bind:value={model}
				class="input text-sm w-full"
			>
				<option value="">Not set (use default)</option>
				{#each CLAUDE_MODELS as m}
					<option value={m.value}>{m.label} — {m.description}</option>
				{/each}
			</select>
		</div>

		<!-- Available Models -->
		<div>
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
				Available Models
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
				Restrict which models can be selected. Leave empty for no restriction.
			</p>
			<div class="flex flex-wrap gap-2">
				{#each AVAILABLE_MODEL_SHORTCUTS as shortcut}
					{@const isSelected = availableModels.includes(shortcut.value)}
					<button
						onclick={() => toggleAvailableModel(shortcut.value)}
						class="px-3 py-1.5 text-sm rounded-md border transition-colors
							{isSelected
							? 'bg-primary-50 dark:bg-primary-900/30 border-primary-300 dark:border-primary-700 text-primary-700 dark:text-primary-300'
							: 'border-gray-200 dark:border-gray-600 text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-500'}"
					>
						{shortcut.label}
					</button>
				{/each}
			</div>
		</div>

		<!-- Output Style -->
		<div>
			<label
				for="output-style-select"
				class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
			>
				Output Style
			</label>
			<select
				id="output-style-select"
				bind:value={outputStyle}
				class="input text-sm w-full"
			>
				{#each OUTPUT_STYLES as style}
					<option value={style.value}>{style.label}</option>
				{/each}
			</select>
		</div>

		<!-- Language -->
		<div>
			<label
				for="language-select"
				class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
			>
				Response Language
			</label>
			<select
				id="language-select"
				bind:value={language}
				class="input text-sm w-full"
			>
				{#each COMMON_LANGUAGES as lang}
					<option value={lang.value}>{lang.label}</option>
				{/each}
			</select>
		</div>

		<!-- Extended Thinking -->
		<div>
			<label
				for="thinking-select"
				class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
			>
				Extended Thinking
			</label>
			<select
				id="thinking-select"
				value={alwaysThinkingEnabled === undefined ? '' : String(alwaysThinkingEnabled)}
				onchange={handleThinkingChange}
				class="input text-sm w-full"
			>
				<option value="">Not set (use default)</option>
				<option value="true">Always enabled</option>
				<option value="false">Disabled</option>
			</select>
		</div>
	</div>

	<div class="mt-5 flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Model Settings
		</button>
	</div>
</div>
