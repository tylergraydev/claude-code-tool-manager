<script lang="ts">
	import type { CreateRuleRequest, Rule } from '$lib/types';
	import { Clipboard, Check, AlertCircle, FileUp } from 'lucide-svelte';

	const MAX_NAME_LENGTH = 64;
	const NAME_PATTERN = /^[a-z0-9-]+$/;

	type Props = {
		initialValues?: Partial<Rule>;
		onSubmit: (values: CreateRuleRequest) => void;
		onCancel: () => void;
	};

	let { initialValues = {}, onSubmit, onCancel }: Props = $props();

	// Form state
	let name = $state(initialValues.name ?? '');
	let description = $state(initialValues.description ?? '');
	let content = $state(initialValues.content ?? '');
	let pathsInput = $state(initialValues.paths?.join(', ') ?? '');
	let tagsInput = $state(initialValues.tags?.join(', ') ?? '');

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	// Import state
	let importStatus = $state<'idle' | 'success' | 'error'>('idle');
	let importMessage = $state('');

	function parseFrontmatter(text: string): { name?: string; description?: string; content: string; paths?: string[] } | null {
		if (!text.startsWith('---')) return null;
		const endIdx = text.indexOf('\n---', 3);
		if (endIdx === -1) return null;

		const fm = text.slice(3, endIdx);
		const body = text.slice(endIdx + 4).trim();
		const result: Record<string, string> = {};

		for (const line of fm.split('\n')) {
			const colonIdx = line.indexOf(':');
			if (colonIdx > 0) {
				const key = line.slice(0, colonIdx).trim();
				const value = line.slice(colonIdx + 1).trim();
				if (key && value) result[key] = value;
			}
		}

		return {
			name: result.name,
			description: result.description,
			content: body,
			paths: result.paths?.split(',').map(s => s.trim()).filter(Boolean)
		};
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
				const parsed = parseFrontmatter(text);

				if (parsed) {
					if (parsed.name) name = parsed.name;
					if (parsed.description) description = parsed.description;
					content = parsed.content;
					if (parsed.paths) pathsInput = parsed.paths.join(', ');
					importStatus = 'success';
					importMessage = parsed.name ? `Imported "${parsed.name}"` : 'Content imported';
				} else {
					// No frontmatter — use filename as name, whole content as body
					name = file.name.replace(/\.(md|markdown|txt)$/, '').toLowerCase().replace(/[^a-z0-9-]/g, '-');
					content = text;
					importStatus = 'success';
					importMessage = 'Content imported (no frontmatter)';
				}

				setTimeout(() => { importStatus = 'idle'; importMessage = ''; }, 3000);
			} catch {
				importStatus = 'error';
				importMessage = 'Could not read file';
				setTimeout(() => { importStatus = 'idle'; importMessage = ''; }, 3000);
			}
		};
		input.click();
	}

	async function handlePasteFromClipboard() {
		try {
			const text = await navigator.clipboard.readText();
			const parsed = parseFrontmatter(text);

			if (parsed) {
				if (parsed.name) name = parsed.name;
				if (parsed.description) description = parsed.description;
				content = parsed.content;
				if (parsed.paths) pathsInput = parsed.paths.join(', ');
				importStatus = 'success';
				importMessage = parsed.name ? `Imported "${parsed.name}"` : 'Content imported';
			} else {
				content = text;
				importStatus = 'success';
				importMessage = 'Content pasted (no frontmatter)';
			}

			setTimeout(() => { importStatus = 'idle'; importMessage = ''; }, 3000);
		} catch {
			importStatus = 'error';
			importMessage = 'Could not access clipboard';
			setTimeout(() => { importStatus = 'idle'; importMessage = ''; }, 3000);
		}
	}

	function validate(): boolean {
		errors = {};
		const trimmedName = name.trim();
		const trimmedContent = content.trim();

		if (!trimmedName) {
			errors.name = 'Name is required';
		} else if (trimmedName.length > MAX_NAME_LENGTH) {
			errors.name = `Name must be ${MAX_NAME_LENGTH} characters or less`;
		} else if (!NAME_PATTERN.test(trimmedName)) {
			errors.name = 'Name must contain only lowercase letters, numbers, and hyphens';
		}

		if (!trimmedContent) {
			errors.content = 'Content is required';
		}

		return Object.keys(errors).length === 0;
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (!validate()) return;

		isSubmitting = true;

		const paths = pathsInput
			.split(',')
			.map((p) => p.trim())
			.filter((p) => p.length > 0);

		const tags = tagsInput
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);

		const request: CreateRuleRequest = {
			name: name.trim(),
			description: description.trim() || undefined,
			content: content.trim(),
			paths: paths.length > 0 ? paths : undefined,
			tags: tags.length > 0 ? tags : undefined
		};

		onSubmit(request);
	}
</script>

<form onsubmit={handleSubmit} class="space-y-6">
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
							Paste or import a <code class="px-1 bg-gray-200 dark:bg-gray-700 rounded">.md</code> rule file
						</p>
					{/if}
				</div>
			</div>
			<div class="flex items-center gap-2">
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
			placeholder="typescript-strict"
		/>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Will be saved to <code class="px-1 bg-gray-100 dark:bg-gray-700 rounded">.claude/rules/{name || 'name'}.md</code>
				<span class="text-gray-400 dark:text-gray-500">&nbsp;&middot;&nbsp;Lowercase letters, numbers, and hyphens only</span>
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
			placeholder="When and why this rule applies"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Human-readable summary of what this rule enforces
		</p>
	</div>

	<!-- Paths (glob patterns) -->
	<div>
		<label for="paths" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			File Patterns
		</label>
		<input
			type="text"
			id="paths"
			bind:value={pathsInput}
			class="input mt-1 font-mono text-sm"
			placeholder="src/**/*.ts, tests/**/*.ts"
		/>
		<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
			Comma-separated glob patterns. Rule loads only when working with matching files. Leave empty for always-active rules.
		</p>
	</div>

	<!-- Content -->
	<div>
		<label for="content" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Rule Instructions <span class="text-red-500">*</span>
		</label>
		<textarea
			id="content"
			bind:value={content}
			rows={10}
			class="input mt-1 font-mono text-sm resize-y"
			class:border-red-500={errors.content}
			placeholder={`Instructions that Claude will follow when this rule is active.

Example:
- Always use strict TypeScript (no \`any\` types)
- Prefer named exports over default exports
- Use kebab-case for file names`}
		></textarea>
		{#if errors.content}
			<p class="mt-1 text-sm text-red-500">{errors.content}</p>
		{:else}
			<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
				Markdown instructions loaded into Claude's context when the rule is active
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
			placeholder="typescript, quality, style"
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
			{initialValues.name ? 'Update Rule' : 'Create Rule'}
		</button>
	</div>
</form>
