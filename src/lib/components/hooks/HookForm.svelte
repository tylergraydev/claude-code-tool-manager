<script lang="ts">
	import type { CreateHookRequest, Hook, HookEventType, HookType } from '$lib/types';
	import { HOOK_EVENT_TYPES } from '$lib/types';
	import { Clipboard, Check, AlertCircle, FileUp, Terminal, MessageSquare, Zap, Globe, Bot, ChevronDown, ChevronRight } from 'lucide-svelte';

	type Props = {
		initialValues?: Partial<Hook>;
		templates?: Hook[];
		onSubmit: (values: CreateHookRequest) => void;
		onCancel: () => void;
	};

	let { initialValues = {}, templates = [], onSubmit, onCancel }: Props = $props();

	// Form state
	let description = $state(initialValues.description ?? '');
	let eventType = $state<HookEventType>(initialValues.eventType ?? 'PostToolUse');
	let matcher = $state(initialValues.matcher ?? '');
	let hookType = $state<HookType>(initialValues.hookType ?? 'command');
	let command = $state(initialValues.command ?? '');
	let prompt = $state(initialValues.prompt ?? '');
	let timeout = $state(initialValues.timeout?.toString() ?? '');
	let url = $state(initialValues.url ?? '');
	let headersInput = $state(initialValues.headers ? JSON.stringify(initialValues.headers, null, 2) : '');
	let allowedEnvVarsInput = $state(initialValues.allowedEnvVars?.join(', ') ?? '');
	let ifCondition = $state(initialValues.ifCondition ?? '');
	let statusMessage = $state(initialValues.statusMessage ?? '');
	let once = $state(initialValues.once ?? false);
	let asyncMode = $state(initialValues.asyncMode ?? false);
	let shell = $state(initialValues.shell ?? 'bash');
	let tagsInput = $state(initialValues.tags?.join(', ') ?? '');
	let showAdvanced = $state(false);

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	// Import state
	let importStatus = $state<'idle' | 'success' | 'error'>('idle');
	let importMessage = $state('');

	// Get current event type metadata
	const currentEventMeta = $derived(HOOK_EVENT_TYPES.find((e) => e.value === eventType));

	// Auto-generate hook name from event type and matcher
	function generateName(): string {
		const matcherPart = matcher.trim();
		if (matcherPart) {
			// Replace special chars with hyphens for a clean name
			const cleanMatcher = matcherPart.replace(/[|]/g, '-').replace(/[^a-zA-Z0-9-]/g, '');
			return `${eventType}-${cleanMatcher}`;
		}
		return eventType;
	}

	function applyTemplate(template: Hook) {
		description = template.description ?? '';
		eventType = template.eventType;
		matcher = template.matcher ?? '';
		hookType = template.hookType;
		command = template.command ?? '';
		prompt = template.prompt ?? '';
		timeout = template.timeout?.toString() ?? '';

		importStatus = 'success';
		importMessage = `Applied template "${template.name}"`;
		setTimeout(() => {
			importStatus = 'idle';
			importMessage = '';
		}, 3000);
	}

	function parseJsonHook(text: string): boolean {
		try {
			const data = JSON.parse(text);
			// Try to detect if this is a settings.json hooks format
			if (data.hooks) {
				// This is a full settings.json
				const eventTypes = Object.keys(data.hooks);
				if (eventTypes.length > 0) {
					const firstEvent = eventTypes[0] as HookEventType;
					const hookEntries = data.hooks[firstEvent];
					if (Array.isArray(hookEntries) && hookEntries.length > 0) {
						const entry = hookEntries[0];
						eventType = firstEvent;
						matcher = entry.matcher ?? '';
						if (entry.hooks && entry.hooks.length > 0) {
							const hookAction = entry.hooks[0];
							hookType = hookAction.type ?? 'command';
							command = hookAction.command ?? '';
							prompt = hookAction.prompt ?? '';
							timeout = hookAction.timeout?.toString() ?? '';
						}
						return true;
					}
				}
			}
		} catch {
			// Not JSON, ignore
		}
		return false;
	}

	async function handlePaste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text');
		if (!text) return;

		if (parseJsonHook(text)) {
			e.preventDefault();
			importStatus = 'success';
			importMessage = 'Imported from JSON';
			setTimeout(() => {
				importStatus = 'idle';
				importMessage = '';
			}, 3000);
		}
	}

	async function handlePasteFromClipboard() {
		try {
			const text = await navigator.clipboard.readText();
			if (parseJsonHook(text)) {
				importStatus = 'success';
				importMessage = 'Imported from JSON';
			} else {
				importStatus = 'error';
				importMessage = 'Could not parse as hook JSON';
			}
			setTimeout(() => {
				importStatus = 'idle';
				importMessage = '';
			}, 3000);
		} catch {
			importStatus = 'error';
			importMessage = 'Could not access clipboard';
			setTimeout(() => {
				importStatus = 'idle';
				importMessage = '';
			}, 3000);
		}
	}

	async function handleFileImport() {
		const input = document.createElement('input');
		input.type = 'file';
		input.accept = '.json';
		input.onchange = async (e) => {
			const file = (e.target as HTMLInputElement).files?.[0];
			if (!file) return;

			try {
				const text = await file.text();
				if (parseJsonHook(text)) {
					importStatus = 'success';
					importMessage = 'Imported from file';
				} else {
					importStatus = 'error';
					importMessage = 'Could not parse file as hook JSON';
				}
				setTimeout(() => {
					importStatus = 'idle';
					importMessage = '';
				}, 3000);
			} catch {
				importStatus = 'error';
				importMessage = 'Could not read file';
				setTimeout(() => {
					importStatus = 'idle';
					importMessage = '';
				}, 3000);
			}
		};
		input.click();
	}

	function validate(): boolean {
		errors = {};

		if (hookType === 'command' && !command.trim()) {
			errors.command = 'Command is required for command hooks';
		}

		if (hookType === 'prompt' && !prompt.trim()) {
			errors.prompt = 'Prompt is required for prompt hooks';
		}

		if (hookType === 'http' && !url.trim()) {
			errors.url = 'URL is required for HTTP hooks';
		}

		if (headersInput.trim()) {
			try {
				JSON.parse(headersInput);
			} catch {
				errors.headers = 'Headers must be valid JSON';
			}
		}

		if (timeout && (isNaN(Number(timeout)) || Number(timeout) < 0)) {
			errors.timeout = 'Timeout must be a positive number';
		}

		return Object.keys(errors).length === 0;
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (!validate()) return;

		isSubmitting = true;

		const tags = tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const parsedHeaders = headersInput.trim() ? JSON.parse(headersInput) : undefined;
		const envVars = allowedEnvVarsInput.split(',').map((v) => v.trim()).filter((v) => v.length > 0);

		const request: CreateHookRequest = {
			name: generateName(),
			description: description.trim() || undefined,
			eventType,
			matcher: matcher.trim() || undefined,
			hookType,
			command: hookType === 'command' ? command.trim() : undefined,
			prompt: hookType === 'prompt' ? prompt.trim() : undefined,
			timeout: hookType === 'command' || hookType === 'http' ? (timeout ? Number(timeout) : undefined) : undefined,
			url: hookType === 'http' ? url.trim() : undefined,
			headers: hookType === 'http' ? parsedHeaders : undefined,
			allowedEnvVars: hookType === 'http' && envVars.length > 0 ? envVars : undefined,
			ifCondition: ifCondition.trim() || undefined,
			statusMessage: statusMessage.trim() || undefined,
			once: once || undefined,
			asyncMode: hookType === 'command' && asyncMode ? true : undefined,
			shell: hookType === 'command' && shell !== 'bash' ? shell : undefined,
			tags: tags.length > 0 ? tags : undefined
		};

		onSubmit(request);
	}
</script>

<form onsubmit={handleSubmit} class="space-y-6" onpaste={handlePaste}>
	<!-- Import Section -->
	<div class="p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-xl bg-gray-50 dark:bg-gray-800/50">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-3">
				{#if importStatus === 'success'}
					<div class="w-8 h-8 rounded-lg bg-green-100 dark:bg-green-900/50 flex items-center justify-center">
						<Check class="w-4 h-4 text-green-600 dark:text-green-400" />
					</div>
				{:else if importStatus === 'error'}
					<div class="w-8 h-8 rounded-lg bg-red-100 dark:bg-red-900/50 flex items-center justify-center">
						<AlertCircle class="w-4 h-4 text-red-600 dark:text-red-400" />
					</div>
				{:else}
					<div class="w-8 h-8 rounded-lg bg-gray-200 dark:bg-gray-700 flex items-center justify-center">
						<Clipboard class="w-4 h-4 text-gray-500 dark:text-gray-400" />
					</div>
				{/if}
				<div>
					{#if importStatus !== 'idle'}
						<p class="text-sm font-medium {importStatus === 'success' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
							{importMessage}
						</p>
					{:else}
						<p class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Import from JSON or Template
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">
							Paste settings.json or select a template
						</p>
					{/if}
				</div>
			</div>
			<div class="flex items-center gap-2">
				{#if templates.length > 0}
					<select
						class="input text-sm py-1.5"
						onchange={(e) => {
							const id = Number((e.target as HTMLSelectElement).value);
							const template = templates.find((t) => t.id === id);
							if (template) applyTemplate(template);
							(e.target as HTMLSelectElement).value = '';
						}}
					>
						<option value="">Templates...</option>
						{#each templates as template}
							<option value={template.id}>{template.name}</option>
						{/each}
					</select>
				{/if}
				<button type="button" onclick={handleFileImport} class="btn btn-secondary text-sm">
					<FileUp class="w-4 h-4 mr-1.5" />
					File
				</button>
				<button type="button" onclick={handlePasteFromClipboard} class="btn btn-secondary text-sm">
					<Clipboard class="w-4 h-4 mr-1.5" />
					Paste
				</button>
			</div>
		</div>
	</div>

	<!-- Description -->
	<div>
		<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Description
		</label>
		<input
			type="text"
			id="description"
			bind:value={description}
			class="input mt-1"
			placeholder="What this hook does"
		/>
	</div>

	<!-- Event Type -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
			Event Type <span class="text-red-500">*</span>
		</label>
		<div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
			{#each HOOK_EVENT_TYPES as event}
				<button
					type="button"
					onclick={() => (eventType = event.value)}
					class="flex flex-col items-start p-3 rounded-lg border-2 transition-all duration-200 text-left {eventType === event.value
						? 'border-orange-500 bg-orange-50 dark:bg-orange-900/20'
						: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
				>
					<div class="flex items-center gap-2">
						<Zap class="w-4 h-4 {eventType === event.value ? 'text-orange-500' : 'text-gray-400'}" />
						<span class="font-medium text-sm text-gray-900 dark:text-white">{event.label}</span>
					</div>
					<span class="text-xs text-gray-500 dark:text-gray-400 mt-1">{event.description}</span>
				</button>
			{/each}
		</div>
	</div>

	<!-- Matcher -->
	<div>
		<label for="matcher" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Matcher Pattern
		</label>
		<input
			type="text"
			id="matcher"
			bind:value={matcher}
			class="input mt-1 font-mono"
			placeholder={currentEventMeta?.matcherHint ?? 'Optional pattern to match'}
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{#if currentEventMeta?.matcherHint}
				{currentEventMeta.matcherHint}. Use <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">|</code> for alternatives.
			{:else}
				Optional regex pattern to filter when this hook runs
			{/if}
		</p>
	</div>

	<!-- Hook Type Toggle -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
			Hook Type <span class="text-red-500">*</span>
		</label>
		<div class="grid grid-cols-2 gap-3">
			<button
				type="button"
				onclick={() => (hookType = 'command')}
				class="flex items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 {hookType === 'command'
					? 'border-gray-500 bg-gray-50 dark:bg-gray-800'
					: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center shrink-0">
					<Terminal class="w-5 h-5 text-gray-600 dark:text-gray-400" />
				</div>
				<div class="text-left">
					<div class="font-medium text-gray-900 dark:text-white">Command</div>
					<div class="text-xs text-gray-500 dark:text-gray-400">Run a shell command</div>
				</div>
			</button>

			<button
				type="button"
				onclick={() => (hookType = 'prompt')}
				class="flex items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 {hookType === 'prompt'
					? 'border-violet-500 bg-violet-50 dark:bg-violet-900/20'
					: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div class="w-10 h-10 rounded-lg bg-violet-100 dark:bg-violet-900/50 flex items-center justify-center shrink-0">
					<MessageSquare class="w-5 h-5 text-violet-600 dark:text-violet-400" />
				</div>
				<div class="text-left">
					<div class="font-medium text-gray-900 dark:text-white">Prompt</div>
					<div class="text-xs text-gray-500 dark:text-gray-400">Inject text into conversation</div>
				</div>
			</button>

			<button
				type="button"
				onclick={() => (hookType = 'http')}
				class="flex items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 {hookType === 'http'
					? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
					: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div class="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/50 flex items-center justify-center shrink-0">
					<Globe class="w-5 h-5 text-blue-600 dark:text-blue-400" />
				</div>
				<div class="text-left">
					<div class="font-medium text-gray-900 dark:text-white">HTTP</div>
					<div class="text-xs text-gray-500 dark:text-gray-400">POST to a URL</div>
				</div>
			</button>

			<button
				type="button"
				onclick={() => (hookType = 'agent')}
				class="flex items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 {hookType === 'agent'
					? 'border-green-500 bg-green-50 dark:bg-green-900/20'
					: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div class="w-10 h-10 rounded-lg bg-green-100 dark:bg-green-900/50 flex items-center justify-center shrink-0">
					<Bot class="w-5 h-5 text-green-600 dark:text-green-400" />
				</div>
				<div class="text-left">
					<div class="font-medium text-gray-900 dark:text-white">Agent</div>
					<div class="text-xs text-gray-500 dark:text-gray-400">Spawn a subagent</div>
				</div>
			</button>
		</div>
	</div>

	<!-- Command (for command type) -->
	{#if hookType === 'command'}
		<div>
			<label for="command" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Command <span class="text-red-500">*</span>
			</label>
			<textarea
				id="command"
				bind:value={command}
				rows={4}
				class="input mt-1 font-mono text-sm resize-y"
				class:border-red-500={errors.command}
				placeholder='npx prettier --write "$CLAUDE_FILE_PATHS"'
			></textarea>
			{#if errors.command}
				<p class="mt-1 text-sm text-red-500">{errors.command}</p>
			{:else}
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Available env vars: <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$CLAUDE_TOOL_NAME</code>,
					<code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$CLAUDE_TOOL_INPUT</code>,
					<code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$CLAUDE_FILE_PATHS</code>
				</p>
			{/if}
		</div>

		<!-- Timeout -->
		<div>
			<label for="timeout" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Timeout (seconds)
			</label>
			<input
				type="number"
				id="timeout"
				bind:value={timeout}
				min="0"
				class="input mt-1 w-32"
				class:border-red-500={errors.timeout}
				placeholder="30"
			/>
			{#if errors.timeout}
				<p class="mt-1 text-sm text-red-500">{errors.timeout}</p>
			{:else}
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Maximum time the command can run before being terminated
				</p>
			{/if}
		</div>
	{/if}

	<!-- Prompt (for prompt type) -->
	{#if hookType === 'prompt'}
		<div>
			<label for="prompt" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Prompt Text <span class="text-red-500">*</span>
			</label>
			<textarea
				id="prompt"
				bind:value={prompt}
				rows={6}
				class="input mt-1 font-mono text-sm resize-y"
				class:border-red-500={errors.prompt}
				placeholder="Additional context or instructions to inject..."
			></textarea>
			{#if errors.prompt}
				<p class="mt-1 text-sm text-red-500">{errors.prompt}</p>
			{:else}
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Text that will be injected into the conversation when this hook runs
				</p>
			{/if}
		</div>
	{/if}

	<!-- HTTP fields -->
	{#if hookType === 'http'}
		<div>
			<label for="url" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				URL <span class="text-red-500">*</span>
			</label>
			<input
				type="text"
				id="url"
				bind:value={url}
				class="input mt-1 font-mono text-sm"
				class:border-red-500={errors.url}
				placeholder="https://example.com/hooks/event"
			/>
			{#if errors.url}
				<p class="mt-1 text-sm text-red-500">{errors.url}</p>
			{:else}
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					URL to POST to. Use <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$VARIABLE</code> for env var substitution.
				</p>
			{/if}
		</div>

		<div>
			<label for="headers" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Headers
			</label>
			<textarea
				id="headers"
				bind:value={headersInput}
				rows={3}
				class="input mt-1 font-mono text-sm resize-y"
				class:border-red-500={errors.headers}
				placeholder={'{"Authorization": "Bearer $API_TOKEN"}'}
			></textarea>
			{#if errors.headers}
				<p class="mt-1 text-sm text-red-500">{errors.headers}</p>
			{:else}
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					JSON object of HTTP headers
				</p>
			{/if}
		</div>

		<div>
			<label for="allowed-env-vars" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Allowed Env Vars
			</label>
			<input
				type="text"
				id="allowed-env-vars"
				bind:value={allowedEnvVarsInput}
				class="input mt-1 font-mono text-sm"
				placeholder="API_TOKEN, WEBHOOK_SECRET"
			/>
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Comma-separated env vars allowed in URL and header substitution
			</p>
		</div>

		<!-- Timeout for HTTP -->
		<div>
			<label for="timeout" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Timeout (seconds)
			</label>
			<input
				type="number"
				id="timeout"
				bind:value={timeout}
				min="0"
				class="input mt-1 w-32"
				class:border-red-500={errors.timeout}
				placeholder="30"
			/>
			{#if errors.timeout}
				<p class="mt-1 text-sm text-red-500">{errors.timeout}</p>
			{/if}
		</div>
	{/if}

	<!-- Advanced Options -->
	<div>
		<button
			type="button"
			onclick={() => (showAdvanced = !showAdvanced)}
			class="flex items-center gap-2 text-sm font-medium text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 transition-colors"
		>
			{#if showAdvanced}
				<ChevronDown class="w-4 h-4" />
			{:else}
				<ChevronRight class="w-4 h-4" />
			{/if}
			Advanced Options
		</button>

		{#if showAdvanced}
			<div class="mt-3 space-y-4 pl-4 border-l-2 border-gray-200 dark:border-gray-700">
				<div>
					<label for="if-condition" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						If Condition
					</label>
					<input
						type="text"
						id="if-condition"
						bind:value={ifCondition}
						class="input mt-1 font-mono text-sm"
						placeholder="Bash(rm *)"
					/>
					<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
						Permission rule syntax filter — hook only runs when this pattern matches
					</p>
				</div>

				<div>
					<label for="status-message" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
						Status Message
					</label>
					<input
						type="text"
						id="status-message"
						bind:value={statusMessage}
						class="input mt-1"
						placeholder="Running formatter..."
					/>
					<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
						Custom spinner text shown while the hook runs
					</p>
				</div>

				<div class="flex items-center gap-6">
					<label class="flex items-center gap-2 cursor-pointer">
						<input type="checkbox" bind:checked={once} class="rounded border-gray-300 dark:border-gray-600" />
						<span class="text-sm text-gray-700 dark:text-gray-300">Run once per session</span>
					</label>

					{#if hookType === 'command'}
						<label class="flex items-center gap-2 cursor-pointer">
							<input type="checkbox" bind:checked={asyncMode} class="rounded border-gray-300 dark:border-gray-600" />
							<span class="text-sm text-gray-700 dark:text-gray-300">Run in background</span>
						</label>
					{/if}
				</div>

				{#if hookType === 'command'}
					<div>
						<label for="shell" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
							Shell
						</label>
						<select id="shell" bind:value={shell} class="input mt-1 w-48">
							<option value="bash">Bash (default)</option>
							<option value="powershell">PowerShell</option>
						</select>
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<!-- Tags -->
	<div>
		<label for="tags" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Tags
		</label>
		<input
			type="text"
			id="tags"
			bind:value={tagsInput}
			class="input mt-1"
			placeholder="formatting, security, logging"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">Comma-separated tags for organization</p>
	</div>

	<!-- Actions -->
	<div class="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
		<button type="button" onclick={onCancel} class="btn btn-secondary">
			Cancel
		</button>
		<button type="submit" class="btn btn-primary" disabled={isSubmitting}>
			{initialValues.name ? 'Update Hook' : 'Create Hook'}
		</button>
	</div>
</form>
