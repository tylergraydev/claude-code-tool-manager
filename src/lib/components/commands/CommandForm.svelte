<script lang="ts">
	import type { CreateCommandRequest, Command } from '$lib/types';
	import { parseSkillMarkdown } from '$lib/utils/markdownParser';
	import { Clipboard, Check, AlertCircle, FileUp, TriangleAlert } from 'lucide-svelte';

	// Validation constants (matching official Claude Code documentation)
	const MAX_NAME_LENGTH = 64;
	const MAX_DESCRIPTION_LENGTH = 1024;
	const RECOMMENDED_MAX_CONTENT_LINES = 500;
	const RESERVED_WORDS = ['anthropic', 'claude'];
	const NAME_PATTERN = /^[a-z0-9-]+$/;

	type Props = {
		initialValues?: Partial<Command>;
		onSubmit: (values: CreateCommandRequest) => void;
		onCancel: () => void;
	};

	let { initialValues = {}, onSubmit, onCancel }: Props = $props();

	// Form state
	let name = $state(initialValues.name ?? '');
	let description = $state(initialValues.description ?? '');
	let content = $state(initialValues.content ?? '');
	let allowedToolsInput = $state(initialValues.allowedTools?.join(', ') ?? '');
	let argumentHint = $state(initialValues.argumentHint ?? '');
	let model = $state(initialValues.model ?? '');
	let tagsInput = $state(initialValues.tags?.join(', ') ?? '');

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});
	let warnings = $state<Record<string, string>>({});

	// Import state
	let importStatus = $state<'idle' | 'success' | 'error'>('idle');
	let importMessage = $state('');

	function applyParsedCommand(parsed: { name?: string; description?: string; content: string; allowedTools?: string[]; argumentHint?: string; model?: string; tags?: string[] }) {
		if (parsed.name) name = parsed.name;
		if (parsed.description) description = parsed.description;
		content = parsed.content;
		if (parsed.allowedTools) allowedToolsInput = parsed.allowedTools.join(', ');
		if (parsed.argumentHint) argumentHint = parsed.argumentHint;
		if (parsed.model) model = parsed.model;
		if (parsed.tags) tagsInput = parsed.tags.join(', ');

		importStatus = 'success';
		importMessage = parsed.name ? `Imported "${parsed.name}"` : 'Content imported';

		setTimeout(() => {
			importStatus = 'idle';
			importMessage = '';
		}, 3000);
	}

	async function handlePaste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text');
		if (!text) return;

		const result = parseSkillMarkdown(text);

		if (result.success && result.data) {
			// Only prevent default if we successfully parsed frontmatter (has name)
			if (result.data.name) {
				e.preventDefault();
				applyParsedCommand(result.data);
			}
		}
	}

	async function handlePasteFromClipboard() {
		try {
			const text = await navigator.clipboard.readText();
			const result = parseSkillMarkdown(text);

			if (result.success && result.data) {
				applyParsedCommand(result.data);
			} else {
				importStatus = 'error';
				importMessage = result.error ?? 'Could not parse clipboard content';
				setTimeout(() => {
					importStatus = 'idle';
					importMessage = '';
				}, 3000);
			}
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
		input.accept = '.md,.markdown,.txt';
		input.onchange = async (e) => {
			const file = (e.target as HTMLInputElement).files?.[0];
			if (!file) return;

			try {
				const text = await file.text();
				const result = parseSkillMarkdown(text);

				if (result.success && result.data) {
					applyParsedCommand(result.data);
				} else {
					importStatus = 'error';
					importMessage = result.error ?? 'Could not parse file';
					setTimeout(() => {
						importStatus = 'idle';
						importMessage = '';
					}, 3000);
				}
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
		warnings = {};
		const trimmedName = name.trim();
		const trimmedDescription = description.trim();
		const trimmedContent = content.trim();

		// Validate name
		if (!trimmedName) {
			errors.name = 'Name is required';
		} else if (trimmedName.length > MAX_NAME_LENGTH) {
			errors.name = `Name must be ${MAX_NAME_LENGTH} characters or less (currently ${trimmedName.length})`;
		} else if (!NAME_PATTERN.test(trimmedName)) {
			errors.name = 'Name must contain only lowercase letters, numbers, and hyphens';
		} else if (trimmedName.includes('<') || trimmedName.includes('>')) {
			errors.name = 'Name cannot contain XML tags (< or >)';
		} else {
			// Check for reserved words
			const nameLower = trimmedName.toLowerCase();
			for (const reserved of RESERVED_WORDS) {
				if (nameLower.includes(reserved)) {
					errors.name = `Name cannot contain reserved word "${reserved}"`;
					break;
				}
			}
		}

		// Validate description
		if (trimmedDescription) {
			if (trimmedDescription.length > MAX_DESCRIPTION_LENGTH) {
				errors.description = `Description must be ${MAX_DESCRIPTION_LENGTH} characters or less (currently ${trimmedDescription.length})`;
			} else if (trimmedDescription.includes('<') || trimmedDescription.includes('>')) {
				errors.description = 'Description cannot contain XML tags (< or >)';
			}
		}

		// Validate content
		if (!trimmedContent) {
			errors.content = 'Content is required';
		} else {
			// Check line count and warn if exceeding recommendation
			const lineCount = trimmedContent.split('\n').length;
			if (lineCount > RECOMMENDED_MAX_CONTENT_LINES) {
				warnings.content = `Content has ${lineCount} lines, exceeding the recommended ${RECOMMENDED_MAX_CONTENT_LINES} lines.`;
			}
		}

		return Object.keys(errors).length === 0;
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (!validate()) return;

		isSubmitting = true;

		const allowedTools = allowedToolsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const tags = tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const request: CreateCommandRequest = {
			name: name.trim(),
			description: description.trim() || undefined,
			content: content.trim(),
			allowedTools: allowedTools.length > 0 ? allowedTools : undefined,
			argumentHint: argumentHint.trim() || undefined,
			model: model.trim() || undefined,
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
							Import from Markdown
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">
							Paste or import a <code class="px-1 bg-gray-200 dark:bg-gray-700 rounded">.md</code> file with YAML frontmatter
						</p>
					{/if}
				</div>
			</div>
			<div class="flex items-center gap-2">
				<button
					type="button"
					onclick={handleFileImport}
					class="btn btn-secondary text-sm"
				>
					<FileUp class="w-4 h-4 mr-1.5" />
					File
				</button>
				<button
					type="button"
					onclick={handlePasteFromClipboard}
					class="btn btn-secondary text-sm"
				>
					<Clipboard class="w-4 h-4 mr-1.5" />
					Paste
				</button>
			</div>
		</div>
	</div>

	<!-- Name -->
	<div>
		<label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Name <span class="text-red-500">*</span>
		</label>
		<div class="relative mt-1">
			<span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">/</span>
			<input
				type="text"
				id="name"
				bind:value={name}
				class="input pl-7"
				class:border-red-500={errors.name}
				placeholder="my-command"
			/>
		</div>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				This will create the command <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">/{name || 'name'}</code>
				<span class="text-gray-400 dark:text-gray-500">&nbsp;·&nbsp;Lowercase letters, numbers, and hyphens only (max {MAX_NAME_LENGTH} chars)</span>
			</p>
		{/if}
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
			class:border-red-500={errors.description}
			placeholder="Brief description of what this command does"
		/>
		{#if errors.description}
			<p class="mt-1 text-sm text-red-500">{errors.description}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Shown in command hints (max {MAX_DESCRIPTION_LENGTH} chars)
			</p>
		{/if}
	</div>

	<!-- Allowed Tools -->
	<div>
		<label for="allowedTools" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Allowed Tools
		</label>
		<input
			type="text"
			id="allowedTools"
			bind:value={allowedToolsInput}
			class="input mt-1"
			placeholder="Read, Edit, Bash(git:*), Glob, Grep"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated list of tools. Use <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">*</code> for all tools.
			Supports patterns like <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">Bash(git:*)</code>
		</p>
	</div>

	<!-- Argument Hint -->
	<div>
		<label for="argumentHint" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Argument Hint
		</label>
		<input
			type="text"
			id="argumentHint"
			bind:value={argumentHint}
			class="input mt-1"
			placeholder="[file] [--verbose] [--dry-run]"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Shows user expected arguments format. Use <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$ARGUMENTS</code> in content to receive them.
		</p>
	</div>

	<!-- Model Selection -->
	<div>
		<label for="model" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Model Override
		</label>
		<select
			id="model"
			bind:value={model}
			class="input mt-1"
		>
			<option value="">Default (inherit from session)</option>
			<option value="opus">Opus (Most capable)</option>
			<option value="sonnet">Sonnet (Balanced)</option>
			<option value="haiku">Haiku (Fast & efficient)</option>
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Optionally force a specific model when executing this command
		</p>
	</div>

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Command Prompt <span class="text-red-500">*</span>
		</label>
		<textarea
			id="content"
			bind:value={content}
			rows={12}
			class="input mt-1 font-mono text-sm resize-y"
			class:border-red-500={errors.content}
			placeholder={`# My Command

This is the prompt content that will be used when the command is invoked.

You can use markdown formatting here.

$ARGUMENTS will be replaced with user input.`}
		></textarea>
		{#if errors.content}
			<p class="mt-1 text-sm text-red-500">{errors.content}</p>
		{:else}
			{#if warnings.content}
				<div class="mt-2 p-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
					<div class="flex items-start gap-2">
						<TriangleAlert class="w-4 h-4 text-amber-600 dark:text-amber-400 mt-0.5 shrink-0" />
						<p class="text-sm text-amber-700 dark:text-amber-300">{warnings.content}</p>
					</div>
				</div>
			{/if}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Use <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$ARGUMENTS</code> to include user-provided arguments
			</p>
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
			placeholder="utility, git, deployment"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated tags for organization
		</p>
	</div>

	<!-- Actions -->
	<div class="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
		<button type="button" onclick={onCancel} class="btn btn-secondary">
			Cancel
		</button>
		<button type="submit" class="btn btn-primary" disabled={isSubmitting}>
			{initialValues.name ? 'Update Command' : 'Create Command'}
		</button>
	</div>
</form>
