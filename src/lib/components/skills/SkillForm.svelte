<script lang="ts">
	import type { CreateSkillRequest, Skill, SkillType } from '$lib/types';
	import { parseSkillMarkdown, type ParsedSkill } from '$lib/utils/markdownParser';
	import { Clipboard, Check, AlertCircle, FileUp, Terminal, Sparkles } from 'lucide-svelte';

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
	let skillType = $state<SkillType>(initialValues.skillType ?? 'command');
	let allowedToolsInput = $state(initialValues.allowedTools?.join(', ') ?? '');
	let argumentHint = $state(initialValues.argumentHint ?? '');
	let model = $state(initialValues.model ?? '');
	let disableModelInvocation = $state(initialValues.disableModelInvocation ?? false);
	let tagsInput = $state(initialValues.tags?.join(', ') ?? '');

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	// Import state
	let importStatus = $state<'idle' | 'success' | 'error'>('idle');
	let importMessage = $state('');

	function applyParsedSkill(skill: ParsedSkill) {
		if (skill.name) name = skill.name;
		if (skill.description) description = skill.description;
		content = skill.content;
		if (skill.skillType) skillType = skill.skillType;
		if (skill.allowedTools) allowedToolsInput = skill.allowedTools.join(', ');
		if (skill.argumentHint) argumentHint = skill.argumentHint;
		if (skill.model) model = skill.model;
		if (skill.disableModelInvocation !== undefined) disableModelInvocation = skill.disableModelInvocation;
		if (skill.tags) tagsInput = skill.tags.join(', ');

		importStatus = 'success';
		importMessage = skill.name ? `Imported "${skill.name}"` : 'Content imported';

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

		if (!name.trim()) {
			errors.name = 'Name is required';
		} else if (!/^[a-zA-Z0-9_-]+$/.test(name.trim())) {
			errors.name = 'Name can only contain letters, numbers, hyphens, and underscores';
		}

		if (!content.trim()) {
			errors.content = 'Content is required';
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

		const request: CreateSkillRequest = {
			name: name.trim(),
			description: description.trim() || undefined,
			content: content.trim(),
			skillType,
			allowedTools: allowedTools.length > 0 ? allowedTools : undefined,
			argumentHint: skillType === 'command' && argumentHint.trim() ? argumentHint.trim() : undefined,
			model: model.trim() || undefined,
			disableModelInvocation: disableModelInvocation || undefined,
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

	<!-- Skill Type Toggle -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
			Skill Type
		</label>
		<div class="flex gap-3">
			<button
				type="button"
				onclick={() => skillType = 'command'}
				class="flex-1 flex items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 {skillType === 'command' ? 'border-amber-500 bg-amber-50 dark:bg-amber-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div class="w-10 h-10 rounded-lg bg-amber-100 dark:bg-amber-900/50 flex items-center justify-center">
					<Terminal class="w-5 h-5 text-amber-600 dark:text-amber-400" />
				</div>
				<div class="text-left">
					<div class="font-medium text-gray-900 dark:text-white">Slash Command</div>
					<div class="text-xs text-gray-500 dark:text-gray-400">User-invoked via /name</div>
				</div>
			</button>

			<button
				type="button"
				onclick={() => skillType = 'skill'}
				class="flex-1 flex items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 {skillType === 'skill' ? 'border-purple-500 bg-purple-50 dark:bg-purple-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
			>
				<div class="w-10 h-10 rounded-lg bg-purple-100 dark:bg-purple-900/50 flex items-center justify-center">
					<Sparkles class="w-5 h-5 text-purple-600 dark:text-purple-400" />
				</div>
				<div class="text-left">
					<div class="font-medium text-gray-900 dark:text-white">Agent Skill</div>
					<div class="text-xs text-gray-500 dark:text-gray-400">Auto-invoked by model</div>
				</div>
			</button>
		</div>
	</div>

	<!-- Name -->
	<div>
		<label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Name <span class="text-red-500">*</span>
		</label>
		<div class="relative mt-1">
			{#if skillType === 'command'}
				<span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">/</span>
			{/if}
			<input
				type="text"
				id="name"
				bind:value={name}
				class="input {skillType === 'command' ? 'pl-7' : ''}"
				class:border-red-500={errors.name}
				placeholder={skillType === 'command' ? 'my-command' : 'my-skill'}
			/>
		</div>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else if skillType === 'command'}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				This will create the command <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">/{name || 'name'}</code>
			</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Will be saved to <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">.claude/skills/{name || 'name'}/SKILL.md</code>
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
			placeholder={skillType === 'command' ? 'Brief description of what this command does' : 'When Claude should invoke this skill'}
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{skillType === 'command' ? 'Shown in command hints' : 'Claude uses this to decide when to invoke the skill'}
		</p>
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

	<!-- Argument Hint (Command only) -->
	{#if skillType === 'command'}
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
	{/if}

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
			Optionally force a specific model when executing this {skillType === 'command' ? 'command' : 'skill'}
		</p>
	</div>

	<!-- Disable Model Invocation (Skill only) -->
	{#if skillType === 'skill'}
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
	{/if}

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{skillType === 'command' ? 'Command Prompt' : 'Skill Instructions'} <span class="text-red-500">*</span>
		</label>
		<textarea
			id="content"
			bind:value={content}
			rows={12}
			class="input mt-1 font-mono text-sm resize-y"
			class:border-red-500={errors.content}
			placeholder={skillType === 'command'
				? `# My Command

This is the prompt content that will be used when the command is invoked.

You can use markdown formatting here.

$ARGUMENTS will be replaced with user input.`
				: `# My Skill

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
		{:else if skillType === 'command'}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Use <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">$ARGUMENTS</code> to include user-provided arguments
			</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Instructions that tell Claude how to execute this skill
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
			placeholder="utility, code-review, testing"
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
