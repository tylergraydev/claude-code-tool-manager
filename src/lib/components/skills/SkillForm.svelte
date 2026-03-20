<script lang="ts">
	import type { CreateSkillRequest, Skill } from '$lib/types';
	import { parseSkillMarkdown } from '$lib/utils/markdownParser';
	import { Clipboard, Check, AlertCircle, FileUp, TriangleAlert } from 'lucide-svelte';
	import { i18n } from '$lib/i18n';

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

		importStatus = 'success';
		importMessage = parsed.name ? i18n.t('commandForm.imported', { name: parsed.name }) : i18n.t('commandForm.contentImported');

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
				importMessage = result.error ?? i18n.t('commandForm.clipboardParseError');
				setTimeout(() => {
					importStatus = 'idle';
					importMessage = '';
				}, 3000);
			}
		} catch {
			importStatus = 'error';
			importMessage = i18n.t('commandForm.clipboardError');
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
					importMessage = result.error ?? i18n.t('commandForm.fileParseError');
					setTimeout(() => {
						importStatus = 'idle';
						importMessage = '';
					}, 3000);
				}
			} catch {
				importStatus = 'error';
				importMessage = i18n.t('commandForm.fileReadError');
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
			errors.name = i18n.t('commandForm.nameRequired');
		} else if (trimmedName.length > MAX_NAME_LENGTH) {
			errors.name = i18n.t('commandForm.nameTooLong', { max: MAX_NAME_LENGTH });
		} else if (!NAME_PATTERN.test(trimmedName)) {
			errors.name = i18n.t('commandForm.nameInvalid');
		} else if (trimmedName.includes('<') || trimmedName.includes('>')) {
			errors.name = i18n.t('commandForm.nameNoXml');
		} else {
			// Check for reserved words
			const nameLower = trimmedName.toLowerCase();
			for (const reserved of RESERVED_WORDS) {
				if (nameLower.includes(reserved)) {
					errors.name = i18n.t('commandForm.nameReserved');
					break;
				}
			}
		}

		// Validate description
		if (trimmedDescription) {
			if (trimmedDescription.length > MAX_DESCRIPTION_LENGTH) {
				errors.description = i18n.t('commandForm.descTooLong', { max: MAX_DESCRIPTION_LENGTH });
			} else if (trimmedDescription.includes('<') || trimmedDescription.includes('>')) {
				errors.description = i18n.t('commandForm.descNoXml');
			}
		}

		// Validate content
		if (!trimmedContent) {
			errors.content = i18n.t('commandForm.contentRequired');
		} else {
			// Check line count and warn if exceeding recommendation
			const lineCount = trimmedContent.split('\n').length;
			if (lineCount > RECOMMENDED_MAX_CONTENT_LINES) {
				warnings.content = i18n.t('commandForm.contentWarning', { lines: lineCount, max: RECOMMENDED_MAX_CONTENT_LINES });
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

		const request: CreateSkillRequest = {
			name: name.trim(),
			description: description.trim() || undefined,
			content: content.trim(),
			allowedTools: allowedTools.length > 0 ? allowedTools : undefined,
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
							{i18n.t('skillForm.importTitle')}
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">
							{i18n.t('commandForm.importDesc')}
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
					{i18n.t('common.file')}
				</button>
				<button
					type="button"
					onclick={handlePasteFromClipboard}
					class="btn btn-secondary text-sm"
				>
					<Clipboard class="w-4 h-4 mr-1.5" />
					{i18n.t('common.paste')}
				</button>
			</div>
		</div>
	</div>

	<!-- Name -->
	<div>
		<label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('common.name')} <span class="text-red-500">*</span>
		</label>
		<input
			type="text"
			id="name"
			bind:value={name}
			class="input mt-1"
			class:border-red-500={errors.name}
			placeholder={i18n.t('skillForm.namePlaceholder')}
		/>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				{i18n.t('skillForm.namePreview', { name: name || 'name' })}
				<span class="text-gray-400 dark:text-gray-500">&nbsp;·&nbsp;{i18n.t('skillForm.nameHelp')}</span>
			</p>
		{/if}
	</div>

	<!-- Description -->
	<div>
		<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('common.description')}
		</label>
		<input
			type="text"
			id="description"
			bind:value={description}
			class="input mt-1"
			class:border-red-500={errors.description}
			placeholder={i18n.t('skillForm.descPlaceholder')}
		/>
		{#if errors.description}
			<p class="mt-1 text-sm text-red-500">{errors.description}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				{i18n.t('skillForm.descHelp', { max: MAX_DESCRIPTION_LENGTH })}
			</p>
		{/if}
	</div>

	<!-- Allowed Tools -->
	<div>
		<label for="allowedTools" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('commandForm.allowedTools')}
		</label>
		<input
			type="text"
			id="allowedTools"
			bind:value={allowedToolsInput}
			class="input mt-1"
			placeholder={i18n.t('commandForm.allowedToolsPlaceholder')}
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('commandForm.allowedToolsHelp')}
		</p>
	</div>

	<!-- Model Selection -->
	<div>
		<label for="model" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('commandForm.modelOverride')}
		</label>
		<select
			id="model"
			bind:value={model}
			class="input mt-1"
		>
			<option value="">{i18n.t('commandForm.modelDefault')}</option>
			<option value="opus">{i18n.t('commandForm.modelOpus')}</option>
			<option value="sonnet">{i18n.t('commandForm.modelSonnet')}</option>
			<option value="haiku">{i18n.t('commandForm.modelHaiku')}</option>
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('commandForm.modelHelp')}
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
				{i18n.t('skillForm.disableInvocation')}
			</label>
			<p class="text-xs text-gray-500 dark:text-gray-400">
				{i18n.t('skillForm.disableInvocationHelp')}
			</p>
		</div>
	</div>

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('skillForm.skillInstructions')} <span class="text-red-500">*</span>
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
			{i18n.t('common.tags')}
		</label>
		<input
			type="text"
			id="tags"
			bind:value={tagsInput}
			class="input mt-1"
			placeholder={i18n.t('skillForm.tagsPlaceholder')}
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('commandForm.tagsHelp')}
		</p>
	</div>

	<!-- Actions -->
	<div class="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
		<button type="button" onclick={onCancel} class="btn btn-secondary">
			{i18n.t('common.cancel')}
		</button>
		<button type="submit" class="btn btn-primary" disabled={isSubmitting}>
			{initialValues.name ? i18n.t('skillForm.updateSkill') : i18n.t('skillForm.createSkill')}
		</button>
	</div>
</form>
