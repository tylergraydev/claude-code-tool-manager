<script lang="ts">
	import type { CreateSubAgentRequest, SubAgent } from '$lib/types';
	import { parseSubAgentMarkdown, type ParsedSubAgent } from '$lib/utils/markdownParser';
	import { Clipboard, Check, AlertCircle, FileUp } from 'lucide-svelte';
	import { i18n } from '$lib/i18n';

	type Props = {
		initialValues?: Partial<SubAgent>;
		onSubmit: (values: CreateSubAgentRequest) => void;
		onCancel: () => void;
	};

	let { initialValues = {}, onSubmit, onCancel }: Props = $props();

	// Form state
	let name = $state(initialValues.name ?? '');
	let description = $state(initialValues.description ?? '');
	let content = $state(initialValues.content ?? '');
	let model = $state(initialValues.model ?? '');
	let permissionMode = $state(initialValues.permissionMode ?? '');
	let toolsInput = $state(initialValues.tools?.join(', ') ?? '');
	let skillsInput = $state(initialValues.skills?.join(', ') ?? '');
	let tagsInput = $state(initialValues.tags?.join(', ') ?? '');

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	// Import state
	let importStatus = $state<'idle' | 'success' | 'error'>('idle');
	let importMessage = $state('');

	function applyParsedSubAgent(subagent: ParsedSubAgent) {
		if (subagent.name) name = subagent.name;
		if (subagent.description) description = subagent.description;
		content = subagent.content;
		if (subagent.tools) toolsInput = subagent.tools.join(', ');
		if (subagent.model) model = subagent.model;
		if (subagent.permissionMode) permissionMode = subagent.permissionMode;
		if (subagent.skills) skillsInput = subagent.skills.join(', ');
		if (subagent.tags) tagsInput = subagent.tags.join(', ');

		importStatus = 'success';
		importMessage = subagent.name ? i18n.t('commandForm.imported', { name: subagent.name }) : i18n.t('commandForm.contentImported');

		setTimeout(() => {
			importStatus = 'idle';
			importMessage = '';
		}, 3000);
	}

	async function handlePaste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text');
		if (!text) return;

		const result = parseSubAgentMarkdown(text);

		if (result.success && result.data) {
			// Only prevent default if we successfully parsed frontmatter (has name)
			if (result.data.name) {
				e.preventDefault();
				applyParsedSubAgent(result.data);
			}
		}
	}

	async function handlePasteFromClipboard() {
		try {
			const text = await navigator.clipboard.readText();
			const result = parseSubAgentMarkdown(text);

			if (result.success && result.data) {
				applyParsedSubAgent(result.data);
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
				const result = parseSubAgentMarkdown(text);

				if (result.success && result.data) {
					applyParsedSubAgent(result.data);
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

	const modelOptions = $derived([
		{ value: '', label: i18n.t('subagentForm.modelDefault') },
		{ value: 'sonnet', label: i18n.t('subagentForm.modelSonnet') },
		{ value: 'opus', label: i18n.t('subagentForm.modelOpus') },
		{ value: 'haiku', label: i18n.t('subagentForm.modelHaiku') },
		{ value: 'inherit', label: i18n.t('subagentForm.modelInherit') }
	]);

	const permissionModeOptions = $derived([
		{ value: '', label: i18n.t('subagentForm.permDefault') },
		{ value: 'default', label: i18n.t('subagentForm.permDefaultShort') },
		{ value: 'acceptEdits', label: i18n.t('subagentForm.permAcceptEdits') },
		{ value: 'dontAsk', label: i18n.t('subagentForm.permDontAsk') },
		{ value: 'bypassPermissions', label: i18n.t('subagentForm.permBypass') },
		{ value: 'plan', label: i18n.t('subagentForm.permPlan') },
		{ value: 'ignore', label: i18n.t('subagentForm.permIgnore') }
	]);

	function validate(): boolean {
		errors = {};

		if (!name.trim()) {
			errors.name = i18n.t('subagentForm.nameRequired');
		} else if (!/^[a-z][a-z0-9-]*$/.test(name.trim())) {
			errors.name = i18n.t('subagentForm.nameInvalid');
		}

		if (!description.trim()) {
			errors.description = i18n.t('subagentForm.descRequired');
		}

		if (!content.trim()) {
			errors.content = i18n.t('subagentForm.contentRequired');
		}

		return Object.keys(errors).length === 0;
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (!validate()) return;

		isSubmitting = true;

		const tools = toolsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const skills = skillsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const tags = tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const request: CreateSubAgentRequest = {
			name: name.trim(),
			description: description.trim(),
			content: content.trim(),
			model: model || undefined,
			permissionMode: permissionMode || undefined,
			tools: tools.length > 0 ? tools : undefined,
			skills: skills.length > 0 ? skills : undefined,
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
							{i18n.t('subagentForm.importTitle')}
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">
							{i18n.t('subagentForm.importDesc')}
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
			placeholder={i18n.t('subagentForm.namePlaceholder')}
		/>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				{i18n.t('subagentForm.nameHelp')}
			</p>
		{/if}
	</div>

	<!-- Description -->
	<div>
		<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('common.description')} <span class="text-red-500">*</span>
		</label>
		<textarea
			id="description"
			bind:value={description}
			rows={2}
			class="input mt-1 resize-none"
			class:border-red-500={errors.description}
			placeholder={i18n.t('subagentForm.descPlaceholder')}
		></textarea>
		{#if errors.description}
			<p class="mt-1 text-sm text-red-500">{errors.description}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				{i18n.t('subagentForm.descHelp')}
			</p>
		{/if}
	</div>

	<!-- Model -->
	<div>
		<label for="model" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('subagentForm.model')}
		</label>
		<select
			id="model"
			bind:value={model}
			class="input mt-1"
		>
			{#each modelOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('subagentForm.modelHelp')}
		</p>
	</div>

	<!-- Permission Mode -->
	<div>
		<label for="permissionMode" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('subagentForm.permissionMode')}
		</label>
		<select
			id="permissionMode"
			bind:value={permissionMode}
			class="input mt-1"
		>
			{#each permissionModeOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('subagentForm.permissionHelp')}
		</p>
	</div>

	<!-- Tools -->
	<div>
		<label for="tools" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('commandForm.allowedTools')}
		</label>
		<input
			type="text"
			id="tools"
			bind:value={toolsInput}
			class="input mt-1"
			placeholder={i18n.t('subagentForm.allowedToolsPlaceholder')}
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('subagentForm.allowedToolsHelp')}
		</p>
	</div>

	<!-- Skills -->
	<div>
		<label for="skills" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('subagentForm.autoLoadSkills')}
		</label>
		<input
			type="text"
			id="skills"
			bind:value={skillsInput}
			class="input mt-1"
			placeholder={i18n.t('subagentForm.autoLoadPlaceholder')}
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			{i18n.t('subagentForm.autoLoadHelp')}
		</p>
	</div>

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			{i18n.t('subagentForm.prompt')} <span class="text-red-500">*</span>
		</label>
		<textarea
			id="content"
			bind:value={content}
			rows={12}
			class="input mt-1 font-mono text-sm resize-y"
			class:border-red-500={errors.content}
			placeholder="You are a specialized sub-agent for...

## Your Responsibilities

1. ...
2. ...

## Guidelines

- ...
- ..."
		></textarea>
		{#if errors.content}
			<p class="mt-1 text-sm text-red-500">{errors.content}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				{i18n.t('subagentForm.promptHelp')}
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
			placeholder={i18n.t('subagentForm.tagsPlaceholder')}
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
			{initialValues.name ? i18n.t('subagentForm.updateAgent') : i18n.t('subagentForm.createAgent')}
		</button>
	</div>
</form>
