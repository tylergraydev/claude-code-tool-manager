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

	// Parse a saved model value into (base, has1m) so the UI can split the [1m] suffix
	// from the underlying ID. The suffix is documented as a Claude Code extension that
	// is stripped before the request reaches the provider.
	function splitModelValue(raw: string | undefined): { base: string; has1m: boolean } {
		if (!raw) return { base: '', has1m: false };
		if (raw.endsWith('[1m]')) return { base: raw.slice(0, -'[1m]'.length), has1m: true };
		return { base: raw, has1m: false };
	}

	const initial = splitModelValue(settings.model);
	const knownValues = CLAUDE_MODELS.map((m) => m.value) as readonly string[];
	const initialIsKnown = !initial.base || knownValues.includes(initial.base);

	let modelChoice = $state<string>(initialIsKnown ? initial.base : '__custom__');
	let customModel = $state<string>(initialIsKnown ? '' : initial.base);
	let use1mContext = $state<boolean>(initial.has1m);
	let availableModels = $state<string[]>([...settings.availableModels]);
	let outputStyle = $state(settings.outputStyle ?? '');
	let language = $state(settings.language ?? '');
	let alwaysThinkingEnabled = $state<boolean | undefined>(settings.alwaysThinkingEnabled);

	// Reset local state when settings prop changes
	$effect(() => {
		const next = splitModelValue(settings.model);
		const known = !next.base || knownValues.includes(next.base);
		modelChoice = known ? next.base : '__custom__';
		customModel = known ? '' : next.base;
		use1mContext = next.has1m;
		availableModels = [...settings.availableModels];
		outputStyle = settings.outputStyle ?? '';
		language = settings.language ?? '';
		alwaysThinkingEnabled = settings.alwaysThinkingEnabled;
	});

	// Resolve the effective model string from the dropdown + custom field + 1m toggle.
	// Returns undefined when nothing is selected so the setting is omitted entirely.
	function resolveModelValue(): string | undefined {
		const base = modelChoice === '__custom__' ? customModel.trim() : modelChoice;
		if (!base) return undefined;
		if (!use1mContext) return base;
		// Don't double-append [1m] if the user typed it themselves
		return base.endsWith('[1m]') ? base : `${base}[1m]`;
	}

	// 1M context only applies to models that support it. Built-in entries declare this
	// via supports1m; custom IDs are assumed eligible (we can't verify) and the user
	// keeps responsibility for typing a valid ID.
	function selectionSupports1m(): boolean {
		if (modelChoice === '__custom__') return customModel.trim().length > 0;
		const entry = CLAUDE_MODELS.find((m) => m.value === modelChoice);
		return entry?.supports1m ?? false;
	}

	// Auto-clear the 1M flag if the user picks a model that doesn't support it
	$effect(() => {
		if (!selectionSupports1m() && use1mContext) {
			use1mContext = false;
		}
	});

	function handleSave() {
		onsave({
			...settings,
			model: resolveModelValue(),
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
				bind:value={modelChoice}
				class="input text-sm w-full"
			>
				<option value="">Not set (use default)</option>
				{#each CLAUDE_MODELS as m}
					<option value={m.value}>{m.label} — {m.description}</option>
				{/each}
				<option value="__custom__">Other (custom model ID)…</option>
			</select>
			{#if modelChoice === '__custom__'}
				<input
					type="text"
					bind:value={customModel}
					placeholder="e.g. claude-opus-4-7 or arn:aws:bedrock:…"
					class="input text-sm w-full mt-2"
					aria-label="Custom model ID"
				/>
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
					Enter any model ID, alias, or provider-specific identifier. Append <code>[1m]</code>
					yourself if not using the toggle below.
				</p>
			{/if}
			<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
				Aliases like <code>opus</code>, <code>sonnet</code>, <code>haiku</code> auto-resolve
				to the latest version. See
				<a
					href="https://code.claude.com/docs/en/model-config#available-models"
					target="_blank"
					rel="noopener"
					class="underline hover:text-primary-600">model configuration docs</a>.
			</p>
		</div>

		<!-- 1M Context Window -->
		<div>
			<label class="flex items-center gap-2 cursor-pointer">
				<input
					type="checkbox"
					bind:checked={use1mContext}
					disabled={!selectionSupports1m()}
					class="rounded border-gray-300 dark:border-gray-600"
				/>
				<span class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Use 1M token context window
				</span>
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400 mt-1 ml-6">
				Appends the <code>[1m]</code> suffix to the model ID. Supported on Opus and Sonnet
				(not Haiku). Standard pricing — no premium beyond 200K tokens.
				<a
					href="https://code.claude.com/docs/en/model-config#extended-context"
					target="_blank"
					rel="noopener"
					class="underline hover:text-primary-600">Docs</a>.
			</p>
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
