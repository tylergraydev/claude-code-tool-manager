<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount, onDestroy } from 'svelte';
	import type {
		Mcp,
		McpTool,
		StartSessionResult,
		ToolCallResult,
		ToolContent,
		ExecutionHistoryEntry
	} from '$lib/types';
	import JsonSchemaForm from './JsonSchemaForm.svelte';
	import {
		X,
		Play,
		Loader2,
		ChevronDown,
		ChevronRight,
		Wrench,
		Clock,
		CheckCircle,
		XCircle,
		Copy,
		Trash2,
		History,
		AlertCircle
	} from 'lucide-svelte';

	type Props = {
		mcp: Mcp;
		onClose: () => void;
	};

	let { mcp, onClose }: Props = $props();

	// Session state
	let sessionId = $state<string | null>(null);
	let tools = $state<McpTool[]>([]);
	let isConnecting = $state(true);
	let connectionError = $state<string | null>(null);

	// Execution state
	let selectedTool = $state<McpTool | null>(null);
	let arguments_ = $state<Record<string, unknown>>({});
	let isExecuting = $state(false);
	let executionResult = $state<ToolCallResult | null>(null);

	// History
	let history = $state<ExecutionHistoryEntry[]>([]);
	let showHistory = $state(false);

	// Tool browser
	let toolSearch = $state('');
	let expandedTools = $state<Set<string>>(new Set());

	const filteredTools = $derived(
		tools.filter(
			(t) =>
				t.name.toLowerCase().includes(toolSearch.toLowerCase()) ||
				t.description?.toLowerCase().includes(toolSearch.toLowerCase())
		)
	);

	onMount(async () => {
		await startSession();
	});

	onDestroy(async () => {
		if (sessionId) {
			try {
				await invoke('end_mcp_session', { sessionId });
			} catch {
				// Ignore cleanup errors
			}
		}
	});

	async function startSession() {
		isConnecting = true;
		connectionError = null;

		try {
			const result = await invoke<StartSessionResult>('start_mcp_session', { mcpId: mcp.id });
			sessionId = result.sessionId;
			tools = result.tools;

			// Auto-select first tool if only one
			if (tools.length === 1) {
				selectTool(tools[0]);
			}
		} catch (e) {
			connectionError = String(e);
		} finally {
			isConnecting = false;
		}
	}

	function selectTool(tool: McpTool) {
		selectedTool = tool;
		arguments_ = {};
		executionResult = null;
	}

	async function executeTool() {
		if (!sessionId || !selectedTool) return;

		isExecuting = true;
		executionResult = null;

		try {
			const result = await invoke<ToolCallResult>('execute_tool', {
				sessionId,
				toolName: selectedTool.name,
				arguments: arguments_
			});

			executionResult = result;

			// Add to history
			const historyEntry: ExecutionHistoryEntry = {
				id: crypto.randomUUID(),
				toolName: selectedTool.name,
				arguments: { ...arguments_ },
				result,
				timestamp: new Date()
			};
			history = [historyEntry, ...history].slice(0, 50); // Keep last 50 entries
		} catch (e) {
			executionResult = {
				success: false,
				content: [],
				isError: true,
				error: String(e),
				executionTimeMs: 0
			};
		} finally {
			isExecuting = false;
		}
	}

	function toggleToolExpansion(toolName: string) {
		const newSet = new Set(expandedTools);
		if (newSet.has(toolName)) {
			newSet.delete(toolName);
		} else {
			newSet.add(toolName);
		}
		expandedTools = newSet;
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
	}

	function clearHistory() {
		history = [];
	}

	function loadFromHistory(entry: ExecutionHistoryEntry) {
		const tool = tools.find((t) => t.name === entry.toolName);
		if (tool) {
			selectTool(tool);
			arguments_ = { ...entry.arguments };
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onClose();
		}
	}

	function renderContent(content: ToolContent): string {
		switch (content.type) {
			case 'text':
				return content.text;
			case 'image':
				return `[Image: ${content.mimeType}]`;
			case 'resource':
				return content.text || `[Resource: ${content.uri}]`;
			default:
				return JSON.stringify(content);
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onclick={handleBackdropClick}
>
	<div
		class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-5xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col"
	>
		<!-- Header -->
		<div
			class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700"
		>
			<div>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Execute: {mcp.name}</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
					{#if sessionId}
						<span class="text-green-600 dark:text-green-400">Connected</span> &bull;
						{tools.length} tools available
					{:else if isConnecting}
						Connecting...
					{:else}
						<span class="text-red-600 dark:text-red-400">Disconnected</span>
					{/if}
				</p>
			</div>
			<div class="flex items-center gap-2">
				<button
					onclick={() => (showHistory = !showHistory)}
					class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
					class:bg-blue-100={showHistory}
					class:dark:bg-blue-900={showHistory}
					class:text-blue-600={showHistory}
					class:dark:text-blue-400={showHistory}
					title="Execution history"
				>
					<History class="w-5 h-5" />
				</button>
				<button
					onclick={onClose}
					class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
				>
					<X class="w-5 h-5" />
				</button>
			</div>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-hidden flex">
			{#if isConnecting}
				<div class="flex-1 flex flex-col items-center justify-center py-12">
					<Loader2 class="w-10 h-10 text-primary-600 animate-spin mb-4" />
					<p class="text-gray-600 dark:text-gray-400">Starting session...</p>
					<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">
						This may take a few seconds
					</p>
				</div>
			{:else if connectionError}
				<div class="flex-1 flex flex-col items-center justify-center py-12 px-8">
					<AlertCircle class="w-12 h-12 text-red-500 mb-4" />
					<p class="text-lg font-medium text-gray-900 dark:text-white mb-2">Connection Failed</p>
					<p class="text-sm text-gray-600 dark:text-gray-400 text-center max-w-md">
						{connectionError}
					</p>
					<button
						onclick={startSession}
						class="mt-4 px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700"
					>
						Retry
					</button>
				</div>
			{:else if showHistory}
				<!-- History Panel -->
				<div class="flex-1 overflow-auto p-4">
					<div class="flex items-center justify-between mb-4">
						<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">
							Execution History ({history.length})
						</h3>
						{#if history.length > 0}
							<button
								onclick={clearHistory}
								class="flex items-center gap-1 px-2 py-1 text-xs text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded"
							>
								<Trash2 class="w-3 h-3" />
								Clear
							</button>
						{/if}
					</div>

					{#if history.length === 0}
						<div class="text-center py-12 text-gray-500 dark:text-gray-400">
							<History class="w-8 h-8 mx-auto mb-2 opacity-50" />
							<p class="text-sm">No execution history yet</p>
						</div>
					{:else}
						<div class="space-y-2">
							{#each history as entry (entry.id)}
								<button
									onclick={() => loadFromHistory(entry)}
									class="w-full text-left p-3 bg-gray-50 dark:bg-gray-900 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800"
								>
									<div class="flex items-center justify-between">
										<div class="flex items-center gap-2">
											{#if entry.result.success}
												<CheckCircle class="w-4 h-4 text-green-500" />
											{:else}
												<XCircle class="w-4 h-4 text-red-500" />
											{/if}
											<span class="font-mono text-sm text-gray-900 dark:text-white">
												{entry.toolName}
											</span>
										</div>
										<span class="text-xs text-gray-500 dark:text-gray-400">
											{entry.timestamp.toLocaleTimeString()}
										</span>
									</div>
									{#if Object.keys(entry.arguments).length > 0}
										<p class="text-xs text-gray-500 dark:text-gray-400 mt-1 truncate">
											{JSON.stringify(entry.arguments)}
										</p>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</div>
			{:else}
				<!-- Tool Browser -->
				<div
					class="w-64 border-r border-gray-200 dark:border-gray-700 overflow-auto flex-shrink-0"
				>
					<div class="p-3 border-b border-gray-200 dark:border-gray-700">
						<input
							type="text"
							bind:value={toolSearch}
							placeholder="Search tools..."
							class="w-full px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
						/>
					</div>

					<div class="py-1">
						{#each filteredTools as tool (tool.name)}
							<button
								onclick={() => selectTool(tool)}
								class="w-full text-left px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-700/50"
								class:bg-blue-50={selectedTool?.name === tool.name}
								class:dark:bg-blue-900={selectedTool?.name === tool.name}
							>
								<div class="flex items-center gap-2">
									<Wrench class="w-4 h-4 text-blue-500 flex-shrink-0" />
									<span class="font-mono text-sm text-gray-900 dark:text-white truncate">
										{tool.name}
									</span>
								</div>
								{#if tool.description}
									<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">
										{tool.description}
									</p>
								{/if}
							</button>
						{/each}

						{#if filteredTools.length === 0}
							<p class="text-center py-4 text-sm text-gray-500 dark:text-gray-400">
								No tools found
							</p>
						{/if}
					</div>
				</div>

				<!-- Tool Execution Panel -->
				<div class="flex-1 overflow-auto p-4">
					{#if selectedTool}
						<div class="space-y-4">
							<!-- Tool Header -->
							<div>
								<div class="flex items-center gap-2">
									<Wrench class="w-5 h-5 text-blue-500" />
									<h3 class="text-lg font-semibold text-gray-900 dark:text-white">
										{selectedTool.name}
									</h3>
								</div>
								{#if selectedTool.description}
									<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
										{selectedTool.description}
									</p>
								{/if}
							</div>

							<!-- Schema Info Toggle -->
							<div>
								<button
									onclick={() => toggleToolExpansion('schema')}
									class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white"
								>
									{#if expandedTools.has('schema')}
										<ChevronDown class="w-4 h-4" />
									{:else}
										<ChevronRight class="w-4 h-4" />
									{/if}
									View input schema
								</button>
								{#if expandedTools.has('schema') && selectedTool.inputSchema}
									<pre
										class="mt-2 p-3 bg-gray-50 dark:bg-gray-900 rounded-lg text-xs overflow-auto max-h-48">{JSON.stringify(selectedTool.inputSchema, null, 2)}</pre>
								{/if}
							</div>

							<!-- Arguments Form -->
							<div class="border-t border-gray-200 dark:border-gray-700 pt-4">
								<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Arguments</h4>
								<JsonSchemaForm
									schema={selectedTool.inputSchema}
									value={arguments_}
									onChange={(v: Record<string, unknown>) => (arguments_ = v)}
									disabled={isExecuting}
								/>
							</div>

							<!-- Execute Button -->
							<div class="flex items-center gap-3">
								<button
									onclick={executeTool}
									disabled={isExecuting}
									class="flex items-center gap-2 px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
								>
									{#if isExecuting}
										<Loader2 class="w-4 h-4 animate-spin" />
										Executing...
									{:else}
										<Play class="w-4 h-4" />
										Execute
									{/if}
								</button>

								{#if Object.keys(arguments_).length > 0}
									<button
										onclick={() => (arguments_ = {})}
										disabled={isExecuting}
										class="px-3 py-2 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
									>
										Clear
									</button>
								{/if}
							</div>

							<!-- Result -->
							{#if executionResult}
								<div
									class="border-t border-gray-200 dark:border-gray-700 pt-4"
								>
									<div class="flex items-center justify-between mb-2">
										<div class="flex items-center gap-2">
											{#if executionResult.success}
												<CheckCircle class="w-5 h-5 text-green-500" />
												<span class="text-sm font-medium text-green-600 dark:text-green-400">
													Success
												</span>
											{:else}
												<XCircle class="w-5 h-5 text-red-500" />
												<span class="text-sm font-medium text-red-600 dark:text-red-400">
													{executionResult.isError ? 'Error' : 'Failed'}
												</span>
											{/if}
											<span class="text-sm text-gray-500 dark:text-gray-400">
												&bull; {executionResult.executionTimeMs}ms
											</span>
										</div>
										<button
											onclick={() =>
												copyToClipboard(
													executionResult!.content
														.map(renderContent)
														.join('\n')
												)}
											class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
											title="Copy result"
										>
											<Copy class="w-4 h-4" />
										</button>
									</div>

									{#if executionResult.error}
										<div
											class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
										>
											<p class="text-sm text-red-700 dark:text-red-300">
												{executionResult.error}
											</p>
										</div>
									{/if}

									{#if executionResult.content.length > 0}
										<div
											class="bg-gray-50 dark:bg-gray-900 rounded-lg p-3 max-h-64 overflow-auto"
										>
											{#each executionResult.content as content, i}
												{#if content.type === 'text'}
													<pre
														class="text-sm text-gray-800 dark:text-gray-200 whitespace-pre-wrap">{content.text}</pre>
												{:else if content.type === 'image'}
													<img
														src={`data:${content.mimeType};base64,${content.data}`}
														alt="Tool output"
														class="max-w-full rounded"
													/>
												{:else if content.type === 'resource'}
													<div class="text-sm">
														<p class="text-gray-500 dark:text-gray-400">
															Resource: {content.uri}
														</p>
														{#if content.text}
															<pre
																class="mt-1 text-gray-800 dark:text-gray-200 whitespace-pre-wrap">{content.text}</pre>
														{/if}
													</div>
												{/if}
												{#if i < executionResult!.content.length - 1}
													<hr class="my-2 border-gray-200 dark:border-gray-700" />
												{/if}
											{/each}
										</div>
									{/if}
								</div>
							{/if}
						</div>
					{:else}
						<div class="flex flex-col items-center justify-center h-full text-gray-500 dark:text-gray-400">
							<Wrench class="w-12 h-12 mb-4 opacity-50" />
							<p class="text-lg font-medium">Select a tool</p>
							<p class="text-sm mt-1">Choose a tool from the list to execute it</p>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-end p-4 border-t border-gray-200 dark:border-gray-700">
			<button onclick={onClose} class="btn btn-secondary"> Close </button>
		</div>
	</div>
</div>
