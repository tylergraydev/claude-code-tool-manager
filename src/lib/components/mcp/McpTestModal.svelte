<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import type { Mcp, McpTestResult } from '$lib/types';
	import { CheckCircle, XCircle, X, RefreshCw, ChevronDown, ChevronRight, Wrench, Database, MessageSquare, Clock } from 'lucide-svelte';

	type Props = {
		mcp: Mcp;
		onClose: () => void;
	};

	let { mcp, onClose }: Props = $props();

	let isLoading = $state(true);
	let result = $state<McpTestResult | null>(null);
	let expandedTools = $state<Set<string>>(new Set());

	onMount(() => {
		runTest();
	});

	async function runTest() {
		isLoading = true;
		result = null;
		expandedTools = new Set();

		try {
			console.log('[MCP Test] Testing MCP id=', mcp.id);
			result = await invoke<McpTestResult>('test_mcp', { mcpId: mcp.id });
			console.log('[MCP Test] Result:', result);
		} catch (e) {
			console.error('[MCP Test] Error:', e);
			result = {
				success: false,
				error: String(e),
				tools: [],
				serverInfo: null,
				resourcesSupported: false,
				promptsSupported: false,
				responseTimeMs: 0
			};
		} finally {
			isLoading = false;
		}
	}

	function toggleTool(toolName: string) {
		const newSet = new Set(expandedTools);
		if (newSet.has(toolName)) {
			newSet.delete(toolName);
		} else {
			newSet.add(toolName);
		}
		expandedTools = newSet;
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onClose();
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
	onclick={handleBackdropClick}
>
	<div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[85vh] overflow-hidden flex flex-col">
		<!-- Header -->
		<div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
			<div>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-white">
					Test MCP: {mcp.name}
				</h2>
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
					{mcp.type === 'stdio' ? mcp.command : mcp.url}
				</p>
			</div>
			<button
				onclick={onClose}
				class="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<X class="w-5 h-5" />
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-auto p-4">
			{#if isLoading}
				<div class="flex flex-col items-center justify-center py-12">
					<div class="animate-spin rounded-full h-10 w-10 border-b-2 border-primary-600 mb-4"></div>
					<p class="text-gray-600 dark:text-gray-400">Testing connection...</p>
					<p class="text-sm text-gray-400 dark:text-gray-500 mt-1">This may take a few seconds</p>
				</div>
			{:else if result}
				{#if result.success}
					<!-- Success State -->
					<div class="space-y-4">
						<!-- Connection Status -->
						<div class="flex items-center gap-3 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
							<CheckCircle class="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0" />
							<div class="flex-1">
								<p class="font-medium text-green-800 dark:text-green-200">
									Connected successfully
								</p>
								<p class="text-sm text-green-600 dark:text-green-400">
									{result.serverInfo?.name || 'Unknown server'}
									{#if result.serverInfo?.version}
										v{result.serverInfo.version}
									{/if}
								</p>
							</div>
							<div class="flex items-center gap-1 text-sm text-green-600 dark:text-green-400">
								<Clock class="w-4 h-4" />
								{result.responseTimeMs}ms
							</div>
						</div>

						<!-- Capabilities -->
						<div>
							<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Capabilities</h3>
							<div class="flex flex-wrap gap-2">
								<div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium
									{result.tools.length > 0
										? 'bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300'
										: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400'}">
									<Wrench class="w-3.5 h-3.5" />
									Tools: {result.tools.length}
								</div>
								<div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium
									{result.resourcesSupported
										? 'bg-purple-100 text-purple-700 dark:bg-purple-900/50 dark:text-purple-300'
										: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400'}">
									<Database class="w-3.5 h-3.5" />
									Resources: {result.resourcesSupported ? 'Yes' : 'No'}
								</div>
								<div class="flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium
									{result.promptsSupported
										? 'bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300'
										: 'bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-400'}">
									<MessageSquare class="w-3.5 h-3.5" />
									Prompts: {result.promptsSupported ? 'Yes' : 'No'}
								</div>
							</div>
						</div>

						<!-- Tools List -->
						{#if result.tools.length > 0}
							<div>
								<h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
									Available Tools ({result.tools.length})
								</h3>
								<div class="space-y-2 max-h-[300px] overflow-auto">
									{#each result.tools as tool (tool.name)}
										<div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
											<button
												onclick={() => toggleTool(tool.name)}
												class="w-full flex items-center gap-2 p-3 text-left hover:bg-gray-50 dark:hover:bg-gray-700/50"
											>
												{#if expandedTools.has(tool.name)}
													<ChevronDown class="w-4 h-4 text-gray-400 flex-shrink-0" />
												{:else}
													<ChevronRight class="w-4 h-4 text-gray-400 flex-shrink-0" />
												{/if}
												<Wrench class="w-4 h-4 text-blue-500 flex-shrink-0" />
												<div class="flex-1 min-w-0">
													<p class="font-mono text-sm font-medium text-gray-900 dark:text-white truncate">
														{tool.name}
													</p>
													{#if tool.description}
														<p class="text-xs text-gray-500 dark:text-gray-400 truncate">
															{tool.description}
														</p>
													{/if}
												</div>
											</button>
											{#if expandedTools.has(tool.name) && tool.inputSchema}
												<div class="px-3 pb-3 pt-0">
													<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-3">
														<p class="text-xs font-medium text-gray-500 dark:text-gray-400 mb-1">Input Schema</p>
														<pre class="text-xs text-gray-700 dark:text-gray-300 overflow-auto max-h-[200px]">{JSON.stringify(tool.inputSchema, null, 2)}</pre>
													</div>
												</div>
											{/if}
										</div>
									{/each}
								</div>
							</div>
						{:else}
							<div class="text-center py-6 text-gray-500 dark:text-gray-400">
								<Wrench class="w-8 h-8 mx-auto mb-2 opacity-50" />
								<p class="text-sm">This MCP doesn't expose any tools</p>
							</div>
						{/if}
					</div>
				{:else}
					<!-- Error State -->
					<div class="space-y-4">
						<div class="flex items-start gap-3 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg border border-red-200 dark:border-red-800">
							<XCircle class="w-5 h-5 text-red-600 dark:text-red-400 flex-shrink-0 mt-0.5" />
							<div class="flex-1 min-w-0">
								<p class="font-medium text-red-800 dark:text-red-200">
									Connection failed
								</p>
								<p class="text-sm text-red-600 dark:text-red-400 mt-1 break-words">
									{result.error}
								</p>
							</div>
						</div>

						<div class="bg-gray-50 dark:bg-gray-900 rounded-lg p-4">
							<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Troubleshooting Tips</h4>
							<ul class="text-sm text-gray-600 dark:text-gray-400 space-y-1.5">
								{#if mcp.type === 'stdio'}
									<li>Make sure the command is installed and available in your PATH</li>
									<li>Check that all required arguments are correctly configured</li>
									<li>Verify any required environment variables are set</li>
								{:else}
									<li>Make sure the URL is accessible from your machine</li>
									<li>Check that the server is running and responding</li>
									<li>Verify any required headers (like API keys) are configured</li>
								{/if}
							</ul>
						</div>
					</div>
				{/if}
			{/if}
		</div>

		<!-- Footer -->
		<div class="flex items-center justify-between p-4 border-t border-gray-200 dark:border-gray-700">
			<button
				onclick={runTest}
				disabled={isLoading}
				class="flex items-center gap-2 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
			>
				<RefreshCw class="w-4 h-4 {isLoading ? 'animate-spin' : ''}" />
				{isLoading ? 'Testing...' : 'Re-run Test'}
			</button>
			<button
				onclick={onClose}
				class="btn btn-secondary"
			>
				Close
			</button>
		</div>
	</div>
</div>
