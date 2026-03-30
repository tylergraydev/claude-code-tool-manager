<script lang="ts">
	import type { CreateSubAgentRequest, SubAgent } from '$lib/types';
	import { parseSubAgentMarkdown, type ParsedSubAgent } from '$lib/utils/markdownParser';
	import { Clipboard, Check, AlertCircle, FileUp } from 'lucide-svelte';

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
	let disallowedToolsInput = $state(initialValues.disallowedTools?.join(', ') ?? '');
	let maxTurns = $state(initialValues.maxTurns?.toString() ?? '');
	let memory = $state(initialValues.memory ?? '');
	let background = $state(initialValues.background ?? false);
	let effort = $state(initialValues.effort ?? '');
	let isolation = $state(initialValues.isolation ?? '');
	let hooksInput = $state(initialValues.hooks ?? '');
	let mcpServersInput = $state(initialValues.mcpServers ?? '');
	let initialPrompt = $state(initialValues.initialPrompt ?? '');

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
		if (subagent.disallowedTools) disallowedToolsInput = subagent.disallowedTools.join(', ');
		if (subagent.maxTurns !== undefined) maxTurns = subagent.maxTurns.toString();
		if (subagent.memory) memory = subagent.memory;
		if (subagent.background !== undefined) background = subagent.background;
		if (subagent.effort) effort = subagent.effort;
		if (subagent.isolation) isolation = subagent.isolation;
		if (subagent.initialPrompt) initialPrompt = subagent.initialPrompt;

		importStatus = 'success';
		importMessage = subagent.name ? `Imported "${subagent.name}"` : 'Content imported';

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
				const result = parseSubAgentMarkdown(text);

				if (result.success && result.data) {
					applyParsedSubAgent(result.data);
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

	const modelOptions = [
		{ value: '', label: 'Default (inherit from parent)' },
		{ value: 'sonnet', label: 'Sonnet' },
		{ value: 'opus', label: 'Opus' },
		{ value: 'haiku', label: 'Haiku' },
		{ value: 'inherit', label: 'Inherit (use main conversation model)' }
	];

	const effortOptions = [
		{ value: '', label: 'Default (inherit from parent)' },
		{ value: 'low', label: 'Low' },
		{ value: 'medium', label: 'Medium' },
		{ value: 'high', label: 'High' },
		{ value: 'max', label: 'Max' }
	];

	const memoryOptions = [
		{ value: '', label: 'None' },
		{ value: 'user', label: 'User (global across projects)' },
		{ value: 'project', label: 'Project (scoped to project)' },
		{ value: 'local', label: 'Local (scoped to working directory)' }
	];

	const isolationOptions = [
		{ value: '', label: 'None (shared workspace)' },
		{ value: 'worktree', label: 'Worktree (isolated git worktree)' }
	];

	const permissionModeOptions = [
		{ value: '', label: 'Default (standard permission prompting)' },
		{ value: 'default', label: 'Default' },
		{ value: 'acceptEdits', label: 'Accept Edits (auto-accepts file edits)' },
		{ value: 'dontAsk', label: "Don't Ask (skip permission prompts)" },
		{ value: 'bypassPermissions', label: 'Bypass Permissions (use with caution)' },
		{ value: 'plan', label: 'Plan (read-only exploration)' },
		{ value: 'ignore', label: 'Ignore (skip this permission mode)' }
	];

	function validate(): boolean {
		errors = {};

		if (!name.trim()) {
			errors.name = 'Name is required';
		} else if (!/^[a-z][a-z0-9-]*$/.test(name.trim())) {
			errors.name = 'Name must start with a lowercase letter and contain only lowercase letters, numbers, and hyphens';
		}

		if (!description.trim()) {
			errors.description = 'Description is required';
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

		const disallowedTools = disallowedToolsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const parsedMaxTurns = maxTurns ? parseInt(maxTurns, 10) : undefined;

		const request: CreateSubAgentRequest = {
			name: name.trim(),
			description: description.trim(),
			content: content.trim(),
			model: model || undefined,
			permissionMode: permissionMode || undefined,
			tools: tools.length > 0 ? tools : undefined,
			skills: skills.length > 0 ? skills : undefined,
			tags: tags.length > 0 ? tags : undefined,
			disallowedTools: disallowedTools.length > 0 ? disallowedTools : undefined,
			maxTurns: parsedMaxTurns && !isNaN(parsedMaxTurns) ? parsedMaxTurns : undefined,
			memory: memory || undefined,
			background: background || undefined,
			effort: effort || undefined,
			isolation: isolation || undefined,
			hooks: hooksInput.trim() || undefined,
			mcpServers: mcpServersInput.trim() || undefined,
			initialPrompt: initialPrompt.trim() || undefined
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
			placeholder="my-sub-agent"
		/>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				This will be the sub-agent's identifier
			</p>
		{/if}
	</div>

	<!-- Description -->
	<div>
		<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Description <span class="text-red-500">*</span>
		</label>
		<textarea
			id="description"
			bind:value={description}
			rows={2}
			class="input mt-1 resize-none"
			class:border-red-500={errors.description}
			placeholder="What this sub-agent does and when to use it"
		></textarea>
		{#if errors.description}
			<p class="mt-1 text-sm text-red-500">{errors.description}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Claude uses this to decide when to delegate to this sub-agent
			</p>
		{/if}
	</div>

	<!-- Model -->
	<div>
		<label for="model" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Model
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
			Optional model override for this sub-agent
		</p>
	</div>

	<!-- Permission Mode -->
	<div>
		<label for="permissionMode" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Permission Mode
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
			Controls how the sub-agent handles permission requests
		</p>
	</div>

	<!-- Tools -->
	<div>
		<label for="tools" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Allowed Tools
		</label>
		<input
			type="text"
			id="tools"
			bind:value={toolsInput}
			class="input mt-1"
			placeholder="Read, Edit, Bash, Glob, Grep"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated list of tools. Leave empty to inherit all tools from parent.
		</p>
	</div>

	<!-- Skills -->
	<div>
		<label for="skills" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Auto-load Skills
		</label>
		<input
			type="text"
			id="skills"
			bind:value={skillsInput}
			class="input mt-1"
			placeholder="commit, review-pr, deploy"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated list of skills to automatically load when sub-agent starts
		</p>
	</div>

	<!-- Disallowed Tools -->
	<div>
		<label for="disallowedTools" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Disallowed Tools
		</label>
		<input
			type="text"
			id="disallowedTools"
			bind:value={disallowedToolsInput}
			class="input mt-1"
			placeholder="Bash, Write, Edit"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated list of tools to deny. Complement of allowed tools.
		</p>
	</div>

	<!-- Max Turns -->
	<div>
		<label for="maxTurns" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Max Turns
		</label>
		<input
			type="number"
			id="maxTurns"
			bind:value={maxTurns}
			class="input mt-1"
			min="1"
			placeholder="e.g. 10"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Maximum number of iterations before the sub-agent stops
		</p>
	</div>

	<!-- Effort -->
	<div>
		<label for="effort" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Effort Level
		</label>
		<select
			id="effort"
			bind:value={effort}
			class="input mt-1"
		>
			{#each effortOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Controls how much thinking the sub-agent does
		</p>
	</div>

	<!-- Memory -->
	<div>
		<label for="memory" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Memory
		</label>
		<select
			id="memory"
			bind:value={memory}
			class="input mt-1"
		>
			{#each memoryOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Persistent memory scope for this sub-agent
		</p>
	</div>

	<!-- Isolation -->
	<div>
		<label for="isolation" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Isolation
		</label>
		<select
			id="isolation"
			bind:value={isolation}
			class="input mt-1"
		>
			{#each isolationOptions as option}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Run the sub-agent in an isolated git worktree
		</p>
	</div>

	<!-- Background -->
	<div class="flex items-center gap-3">
		<input
			type="checkbox"
			id="background"
			bind:checked={background}
			class="h-4 w-4 rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500"
		/>
		<label for="background" class="text-sm font-medium text-gray-700 dark:text-gray-300">
			Always run in background
		</label>
	</div>

	<!-- Initial Prompt -->
	<div>
		<label for="initialPrompt" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Initial Prompt
		</label>
		<textarea
			id="initialPrompt"
			bind:value={initialPrompt}
			rows={2}
			class="input mt-1 resize-none"
			placeholder="Auto-submitted first prompt when the sub-agent starts"
		></textarea>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Automatically submitted as the first prompt when this sub-agent is invoked
		</p>
	</div>

	<!-- Hooks (JSON) -->
	<div>
		<label for="hooks" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Scoped Hooks
		</label>
		<textarea
			id="hooks"
			bind:value={hooksInput}
			rows={3}
			class="input mt-1 font-mono text-sm resize-y"
			placeholder={'{"PreToolUse": [{"command": "echo pre"}]}'}
		></textarea>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			JSON object of hooks that run only within this sub-agent
		</p>
	</div>

	<!-- MCP Servers (JSON) -->
	<div>
		<label for="mcpServers" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Scoped MCP Servers
		</label>
		<textarea
			id="mcpServers"
			bind:value={mcpServersInput}
			rows={3}
			class="input mt-1 font-mono text-sm resize-y"
			placeholder={'{"my-server": {"command": "npx", "args": ["-y", "my-server"]}}'}
		></textarea>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			JSON object of MCP servers available only to this sub-agent
		</p>
	</div>

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Sub-Agent Prompt <span class="text-red-500">*</span>
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
				The system prompt that defines this sub-agent's behavior
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
			{initialValues.name ? 'Update Sub-Agent' : 'Create Sub-Agent'}
		</button>
	</div>
</form>
