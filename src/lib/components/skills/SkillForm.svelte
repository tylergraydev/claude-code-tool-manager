<script lang="ts">
	import type { CreateSkillRequest, Skill } from '$lib/types';
	import { parseSkillMarkdown } from '$lib/utils/markdownParser';
	import { Clipboard, Check, AlertCircle, FileUp, TriangleAlert } from 'lucide-svelte';

	// Validation constants (matching official Claude Code documentation)
	const MAX_NAME_LENGTH = 64;
	const MAX_DESCRIPTION_LENGTH = 1024;
	const RECOMMENDED_MAX_CONTENT_LINES = 500;
	const RESERVED_WORDS = ['anthropic', 'claude'];
	const NAME_PATTERN = /^[a-z0-9-]+$/;

	type Props = {
		initialValues?: Partial<Skill>;
		onSubmit: (values: CreateSkillRequest) => void;
		onCancel: () => void;
	};

	let { initialValues = {}, onSubmit, onCancel }: Props = $props();

	// Form state
	let name = $state(initialValues.name ?? '');
	let description = $state(initialValues.description ?? '');
	let content = $state(initialValues.content ?? '');
	let allowedToolsInput = $state(initialValues.allowedTools?.join(', ') ?? '');
	let model = $state(initialValues.model ?? '');
	let disableModelInvocation = $state(initialValues.disableModelInvocation ?? false);
	let tagsInput = $state(initialValues.tags?.join(', ') ?? '');
	let contextValue = $state(initialValues.context ?? '');
	let agent = $state(initialValues.agent ?? '');
	let hooksInput = $state(initialValues.hooks ?? '');
	let pathsInput = $state(initialValues.paths?.join(', ') ?? '');
	let shell = $state(initialValues.shell ?? '');
	let once = $state(initialValues.once ?? false);
	let effort = $state(initialValues.effort ?? '');

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});
	let warnings = $state<Record<string, string>>({});

	// Import state
	let importStatus = $state<'idle' | 'success' | 'error'>('idle');
	let importMessage = $state('');

	function applyParsedSkill(parsed: { name?: string; description?: string; content: string; allowedTools?: string[]; model?: string; disableModelInvocation?: boolean; tags?: string[] }) {
		if (parsed.name) name = parsed.name;
		if (parsed.description) description = parsed.description;
		content = parsed.content;
		if (parsed.allowedTools) allowedToolsInput = parsed.allowedTools.join(', ');
		if (parsed.model) model = parsed.model;
		if (parsed.disableModelInvocation !== undefined) disableModelInvocation = parsed.disableModelInvocation;
		if (parsed.tags) tagsInput = parsed.tags.join(', ');
		if (parsed.context) contextValue = parsed.context;
		if (parsed.agent) agent = parsed.agent;
		if (parsed.hooks) hooksInput = parsed.hooks;
		if (parsed.paths) pathsInput = parsed.paths.join(', ');
		if (parsed.shell) shell = parsed.shell;
		if (parsed.once !== undefined) once = parsed.once;
		if (parsed.effort) effort = parsed.effort;

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
				applyParsedSkill(result.data);
			}
		}
	}

	async function handlePasteFromClipboard() {
		try {
			const text = await navigator.clipboard.readText();
			const result = parseSkillMarkdown(text);

			if (result.success && result.data) {
				applyParsedSkill(result.data);
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
					applyParsedSkill(result.data);
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
				warnings.content = `Content has ${lineCount} lines, exceeding the recommended ${RECOMMENDED_MAX_CONTENT_LINES} lines. Consider splitting into separate reference files.`;
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

		const paths = pathsInput
			.split(',')
			.map((p) => p.trim())
			.filter((p) => p.length > 0);

		const request: CreateSkillRequest = {
			name: name.trim(),
			description: description.trim() || undefined,
			content: content.trim(),
			allowedTools: allowedTools.length > 0 ? allowedTools : undefined,
			model: model.trim() || undefined,
			disableModelInvocation: disableModelInvocation || undefined,
			tags: tags.length > 0 ? tags : undefined,
			context: contextValue || undefined,
			agent: agent.trim() || undefined,
			hooks: hooksInput.trim() || undefined,
			paths: paths.length > 0 ? paths : undefined,
			shell: shell || undefined,
			once: once || undefined,
			effort: effort || undefined
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
		<input
			type="text"
			id="name"
			bind:value={name}
			class="input mt-1"
			class:border-red-500={errors.name}
			placeholder="my-skill"
		/>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Will be saved to <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">.claude/skills/{name || 'name'}/SKILL.md</code>
				<span class="text-gray-400 dark:text-gray-500">&nbsp;·&nbsp;Lowercase letters, numbers, and hyphens only</span>
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
			placeholder="When Claude should invoke this skill"
		/>
		{#if errors.description}
			<p class="mt-1 text-sm text-red-500">{errors.description}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Claude uses this to decide when to invoke the skill (max {MAX_DESCRIPTION_LENGTH} chars)
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
			Optionally force a specific model when executing this skill
		</p>
	</div>

	<!-- Disable Model Invocation -->
	<div class="flex items-start gap-3">
		<input
			type="checkbox"
			id="disableModelInvocation"
			bind:checked={disableModelInvocation}
			class="mt-1 w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-purple-600 focus:ring-purple-500"
		/>
		<div>
			<label for="disableModelInvocation" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Disable Model Invocation
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400">
				Prevent Claude from automatically invoking this skill. Useful for skills that should only be called by other skills.
			</p>
		</div>
	</div>

	<!-- Context -->
	<div>
		<label for="context" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Context
		</label>
		<select
			id="context"
			bind:value={contextValue}
			class="input mt-1"
		>
			<option value="">Default (inline)</option>
			<option value="fork">Fork (run in subagent)</option>
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Run this skill inline or fork it into a subagent context
		</p>
	</div>

	<!-- Agent (shown when context is fork) -->
	{#if contextValue === 'fork'}
		<div>
			<label for="agent" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Agent
			</label>
			<input
				type="text"
				id="agent"
				bind:value={agent}
				class="input mt-1"
				placeholder="code-reviewer"
			/>
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Which subagent type to use when running in fork context
			</p>
		</div>
	{/if}

	<!-- Shell -->
	<div>
		<label for="shell" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Shell
		</label>
		<select
			id="shell"
			bind:value={shell}
			class="input mt-1"
		>
			<option value="">Default (bash)</option>
			<option value="bash">bash</option>
			<option value="powershell">powershell</option>
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Shell used for command execution within this skill
		</p>
	</div>

	<!-- Effort Level -->
	<div>
		<label for="effort" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Effort Level
		</label>
		<select
			id="effort"
			bind:value={effort}
			class="input mt-1"
		>
			<option value="">Default (inherit from parent)</option>
			<option value="low">Low</option>
			<option value="medium">Medium</option>
			<option value="high">High</option>
			<option value="max">Max</option>
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Controls how much thinking Claude does within this skill
		</p>
	</div>

	<!-- Once -->
	<div class="flex items-start gap-3">
		<input
			type="checkbox"
			id="once"
			bind:checked={once}
			class="mt-1 w-4 h-4 rounded border-gray-300 dark:border-gray-600 text-purple-600 focus:ring-purple-500"
		/>
		<div>
			<label for="once" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
				Run Once Per Session
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400">
				Only execute this skill once per Claude Code session
			</p>
		</div>
	</div>

	<!-- Paths -->
	<div>
		<label for="paths" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Paths
		</label>
		<input
			type="text"
			id="paths"
			bind:value={pathsInput}
			class="input mt-1"
			placeholder="src/**/*.ts, lib/**/*.js"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated glob patterns for auto-loading this skill
		</p>
	</div>

	<!-- Hooks -->
	<div>
		<label for="hooks" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Hooks
		</label>
		<textarea
			id="hooks"
			bind:value={hooksInput}
			rows={4}
			class="input mt-1 font-mono text-sm resize-y"
			placeholder={`{"preToolUse": {"command": "echo pre"}, "postToolUse": {"command": "echo post"}}`}
		></textarea>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			JSON object defining lifecycle hooks scoped to this skill
		</p>
	</div>

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Skill Instructions <span class="text-red-500">*</span>
		</label>
		<textarea
			id="content"
			bind:value={content}
			rows={12}
			class="input mt-1 font-mono text-sm resize-y"
			class:border-red-500={errors.content}
			placeholder={`# My Skill

Instructions for Claude when this skill is invoked.

## When to Use

- When the user asks about...
- When working with...

## How to Execute

1. First...
2. Then...`}
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
				Instructions that tell Claude how to execute this skill (recommended: under {RECOMMENDED_MAX_CONTENT_LINES} lines)
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
			placeholder="code-review, testing, documentation"
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
			{initialValues.name ? 'Update Skill' : 'Create Skill'}
		</button>
	</div>
</form>
