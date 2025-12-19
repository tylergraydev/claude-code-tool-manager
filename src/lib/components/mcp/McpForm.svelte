<script lang="ts">
	import type { CreateMcpRequest, McpType, Mcp } from '$lib/types';
	import McpTypeSelector from './McpTypeSelector.svelte';
	import { EnvEditor } from '$lib/components/shared';
	import { parseMcpFromClipboard, type ParsedMcp } from '$lib/utils/mcpPasteParser';
	import { Clipboard, Check, AlertCircle } from 'lucide-svelte';

	type Props = {
		initialValues?: Partial<Mcp>;
		onSubmit: (values: CreateMcpRequest) => void;
		onCancel: () => void;
	};

	let { initialValues = {}, onSubmit, onCancel }: Props = $props();

	// Form state
	let name = $state(initialValues.name ?? '');
	let description = $state(initialValues.description ?? '');
	let mcpType = $state<McpType>(initialValues.type ?? 'stdio');

	// stdio fields
	let command = $state(initialValues.command ?? '');
	let args = $state(initialValues.args?.join(' ') ?? '');

	// sse/http fields
	let url = $state(initialValues.url ?? '');
	let headers = $state<Record<string, string>>(
		(initialValues.headers as Record<string, string>) ?? {}
	);

	// Common fields
	let env = $state<Record<string, string>>(
		(initialValues.env as Record<string, string>) ?? {}
	);

	let isSubmitting = $state(false);
	let errors = $state<Record<string, string>>({});

	// Paste detection state
	let pasteStatus = $state<'idle' | 'success' | 'error'>('idle');
	let pasteMessage = $state('');

	function applyParsedMcp(mcp: ParsedMcp) {
		name = mcp.name;
		mcpType = mcp.type;

		if (mcp.type === 'stdio') {
			command = mcp.command ?? '';
			args = mcp.args?.join(' ') ?? '';
		} else {
			url = mcp.url ?? '';
			if (mcp.headers) {
				headers = mcp.headers;
			}
		}

		if (mcp.env) {
			env = mcp.env;
		}

		pasteStatus = 'success';
		pasteMessage = `Imported "${mcp.name}" (${mcp.type})`;

		// Reset status after 3 seconds
		setTimeout(() => {
			pasteStatus = 'idle';
			pasteMessage = '';
		}, 3000);
	}

	async function handlePaste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text');
		if (!text) return;

		const result = parseMcpFromClipboard(text);

		if (result.success && result.mcps.length > 0) {
			e.preventDefault(); // Prevent default paste into focused field
			applyParsedMcp(result.mcps[0]);

			// If multiple MCPs were found, notify user
			if (result.mcps.length > 1) {
				pasteMessage += ` (+${result.mcps.length - 1} more available)`;
			}
		}
		// If parsing fails, let the default paste behavior happen
	}

	async function handlePasteFromClipboard() {
		try {
			const text = await navigator.clipboard.readText();
			const result = parseMcpFromClipboard(text);

			if (result.success && result.mcps.length > 0) {
				applyParsedMcp(result.mcps[0]);
			} else {
				pasteStatus = 'error';
				pasteMessage = result.error ?? 'Could not parse clipboard content';
				setTimeout(() => {
					pasteStatus = 'idle';
					pasteMessage = '';
				}, 3000);
			}
		} catch {
			pasteStatus = 'error';
			pasteMessage = 'Could not access clipboard';
			setTimeout(() => {
				pasteStatus = 'idle';
				pasteMessage = '';
			}, 3000);
		}
	}

	function validate(): boolean {
		errors = {};

		if (!name.trim()) {
			errors.name = 'Name is required';
		} else if (!/^[a-zA-Z0-9_-]+$/.test(name.trim())) {
			errors.name = 'Name can only contain letters, numbers, hyphens, and underscores';
		}

		if (mcpType === 'stdio') {
			if (!command.trim()) {
				errors.command = 'Command is required';
			}
		} else {
			if (!url.trim()) {
				errors.url = 'URL is required';
			} else {
				try {
					new URL(url);
				} catch {
					errors.url = 'Invalid URL format';
				}
			}
		}

		return Object.keys(errors).length === 0;
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (!validate()) return;

		isSubmitting = true;

		const request: CreateMcpRequest = {
			name: name.trim(),
			description: description.trim() || undefined,
			type: mcpType,
			command: mcpType === 'stdio' ? command.trim() : undefined,
			args: mcpType === 'stdio' && args.trim() ? args.trim().split(/\s+/) : undefined,
			url: mcpType !== 'stdio' ? url.trim() : undefined,
			headers: mcpType === 'http' && Object.keys(headers).length ? headers : undefined,
			env: Object.keys(env).length ? env : undefined
		};

		onSubmit(request);
	}
</script>

<form onsubmit={handleSubmit} class="space-y-6" onpaste={handlePaste}>
	<!-- Paste Import Section -->
	<div class="p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-xl bg-gray-50 dark:bg-gray-800/50">
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-3">
				{#if pasteStatus === 'success'}
					<div class="w-8 h-8 rounded-lg bg-green-100 dark:bg-green-900/50 flex items-center justify-center">
						<Check class="w-4 h-4 text-green-600 dark:text-green-400" />
					</div>
				{:else if pasteStatus === 'error'}
					<div class="w-8 h-8 rounded-lg bg-red-100 dark:bg-red-900/50 flex items-center justify-center">
						<AlertCircle class="w-4 h-4 text-red-600 dark:text-red-400" />
					</div>
				{:else}
					<div class="w-8 h-8 rounded-lg bg-gray-200 dark:bg-gray-700 flex items-center justify-center">
						<Clipboard class="w-4 h-4 text-gray-500 dark:text-gray-400" />
					</div>
				{/if}
				<div>
					{#if pasteStatus !== 'idle'}
						<p class="text-sm font-medium {pasteStatus === 'success' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
							{pasteMessage}
						</p>
					{:else}
						<p class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Quick Import
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400">
							Paste a <code class="px-1 bg-gray-200 dark:bg-gray-700 rounded">claude mcp add</code> command or JSON config
						</p>
					{/if}
				</div>
			</div>
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
			placeholder="my-mcp-server"
		/>
		{#if errors.name}
			<p class="mt-1 text-sm text-red-500">{errors.name}</p>
		{/if}
	</div>

	<!-- Description -->
	<div>
		<label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
			Description
		</label>
		<textarea
			id="description"
			bind:value={description}
			rows={2}
			class="input mt-1 resize-none"
			placeholder="Optional description of what this MCP does..."
		></textarea>
	</div>

	<!-- Type Selector -->
	<McpTypeSelector bind:value={mcpType} />

	<!-- Type-specific fields -->
	{#if mcpType === 'stdio'}
		<div class="space-y-4 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
			<div>
				<label for="command" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Command <span class="text-red-500">*</span>
				</label>
				<input
					type="text"
					id="command"
					bind:value={command}
					class="input mt-1 font-mono"
					class:border-red-500={errors.command}
					placeholder="npx"
				/>
				{#if errors.command}
					<p class="mt-1 text-sm text-red-500">{errors.command}</p>
				{/if}
			</div>

			<div>
				<label for="args" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					Arguments
				</label>
				<input
					type="text"
					id="args"
					bind:value={args}
					class="input mt-1 font-mono"
					placeholder="-y @package/mcp-server"
				/>
				<p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
					Space-separated arguments
				</p>
			</div>
		</div>
	{:else}
		<div class="space-y-4 p-4 bg-gray-50 dark:bg-gray-800/50 rounded-xl">
			<div>
				<label for="url" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
					URL <span class="text-red-500">*</span>
				</label>
				<input
					type="url"
					id="url"
					bind:value={url}
					class="input mt-1 font-mono"
					class:border-red-500={errors.url}
					placeholder={mcpType === 'sse' ? 'https://mcp.service.com/sse' : 'https://api.service.com/mcp'}
				/>
				{#if errors.url}
					<p class="mt-1 text-sm text-red-500">{errors.url}</p>
				{/if}
			</div>

			{#if mcpType === 'http'}
				<div>
					<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
						Headers
					</label>
					<EnvEditor
						bind:values={headers}
						keyPlaceholder="Header name"
						valuePlaceholder={'Header value (use ${VAR} for env vars)'}
					/>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Environment Variables -->
	<div>
		<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
			Environment Variables
		</label>
		<EnvEditor bind:values={env} />
	</div>

	<!-- Actions -->
	<div class="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
		<button type="button" onclick={onCancel} class="btn btn-secondary">
			Cancel
		</button>
		<button type="submit" class="btn btn-primary" disabled={isSubmitting}>
			{initialValues.name ? 'Update MCP' : 'Create MCP'}
		</button>
	</div>
</form>
