<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { GlobalSettings } from '$lib/components/global';
	import { invoke } from '@tauri-apps/api/core';
	import { notifications, whatsNew } from '$lib/stores';
	import { FolderOpen, FileText, RefreshCw, Sparkles, Check, AlertCircle, Server, Play, Square, Copy, Library, Trash2, Network, RotateCw, Key } from 'lucide-svelte';
	import { getVersion } from '@tauri-apps/api/app';
	import type { GatewayServerConfig, GatewayServerStatus, BackendInfo } from '$lib/types';

	let appVersion = $state('');

	interface ClaudePaths {
		claudeDir: string;
		claudeJson: string;
		globalSettings: string;
		pluginsDir: string;
	}

	interface OpenCodePaths {
		configDir: string;
		configFile: string;
		commandDir: string;
		agentDir: string;
		pluginDir: string;
		toolDir: string;
		knowledgeDir: string;
	}

	interface CodexPaths {
		configDir: string;
		configFile: string;
	}

	interface CopilotPaths {
		configDir: string;
		configFile: string;
		mcpConfigFile: string;
		agentsDir: string;
	}

	interface CursorPaths {
		configDir: string;
		mcpConfigFile: string;
	}

	interface GeminiPaths {
		configDir: string;
		settingsFile: string;
	}

	interface EditorInfo {
		id: string;
		name: string;
		isInstalled: boolean;
		isEnabled: boolean;
		configPath: string;
	}

	interface AppSettings {
		enabledEditors: string[];
	}

	interface McpServerStatus {
		isRunning: boolean;
		port: number;
		url: string;
		mcpEndpoint: string;
	}

	interface McpServerConfig {
		enabled: boolean;
		port: number;
		autoStart: boolean;
	}

	let claudePaths = $state<ClaudePaths | null>(null);
	let opencodePaths = $state<OpenCodePaths | null>(null);
	let codexPaths = $state<CodexPaths | null>(null);
	let copilotPaths = $state<CopilotPaths | null>(null);
	let cursorPaths = $state<CursorPaths | null>(null);
	let geminiPaths = $state<GeminiPaths | null>(null);
	let editors = $state<EditorInfo[]>([]);
	let appSettings = $state<AppSettings>({ enabledEditors: ['claude_code'] });
	let togglingEditor = $state<string | null>(null);

	// MCP Server state
	let mcpServerStatus = $state<McpServerStatus | null>(null);
	let mcpServerConfig = $state<McpServerConfig>({ enabled: true, port: 23847, autoStart: true });
	let isServerLoading = $state(false);
	let isSelfMcpInLibrary = $state(false);

	// GitHub Token state
	let githubToken = $state('');
	let hasToken = $state(false);
	let isSavingToken = $state(false);

	// Gateway state
	let gatewayStatus = $state<GatewayServerStatus | null>(null);
	let gatewayConfig = $state<GatewayServerConfig>({ enabled: false, port: 23848, autoStart: false });
	let isGatewayLoading = $state(false);
	let restartingBackend = $state<number | null>(null);

	async function loadPaths() {
		try {
			claudePaths = await invoke<ClaudePaths>('get_claude_paths');
		} catch (err) {
			console.error('Failed to load Claude paths:', err);
		}
		try {
			opencodePaths = await invoke<OpenCodePaths>('get_opencode_paths_cmd');
		} catch (err) {
			console.error('Failed to load OpenCode paths:', err);
		}
		try {
			codexPaths = await invoke<CodexPaths>('get_codex_paths_cmd');
		} catch (err) {
			console.error('Failed to load Codex paths:', err);
		}
		try {
			copilotPaths = await invoke<CopilotPaths>('get_copilot_paths_cmd');
		} catch (err) {
			console.error('Failed to load Copilot paths:', err);
		}
		try {
			cursorPaths = await invoke<CursorPaths>('get_cursor_paths_cmd');
		} catch (err) {
			console.error('Failed to load Cursor paths:', err);
		}
		try {
			geminiPaths = await invoke<GeminiPaths>('get_gemini_paths_cmd');
		} catch (err) {
			console.error('Failed to load Gemini paths:', err);
		}
	}

	async function loadEditors() {
		try {
			editors = await invoke<EditorInfo[]>('get_available_editors');
		} catch (err) {
			console.error('Failed to load editors:', err);
		}
	}

	async function loadAppSettings() {
		try {
			appSettings = await invoke<AppSettings>('get_app_settings');
		} catch (err) {
			console.error('Failed to load app settings:', err);
		}
	}

	async function toggleEditor(editorId: string, enabled: boolean) {
		togglingEditor = editorId;
		try {
			await invoke('toggle_editor', { editorId, enabled });
			// Update local state
			const editorIndex = editors.findIndex(e => e.id === editorId);
			if (editorIndex >= 0) {
				editors[editorIndex].isEnabled = enabled;
			}
			// Update appSettings
			if (enabled) {
				if (!appSettings.enabledEditors.includes(editorId)) {
					appSettings.enabledEditors = [...appSettings.enabledEditors, editorId];
				}
			} else {
				appSettings.enabledEditors = appSettings.enabledEditors.filter(id => id !== editorId);
			}
			notifications.success(`${enabled ? 'Enabled' : 'Disabled'} ${getEditorDisplayName(editorId)}`);
		} catch (err) {
			notifications.error(`Failed to toggle editor: ${err}`);
		} finally {
			togglingEditor = null;
		}
	}

	async function openConfigFile(path: string) {
		try {
			await invoke('open_config_file', { path });
		} catch (err) {
			notifications.error('Failed to open file');
		}
	}

	async function backupConfigs() {
		try {
			await invoke('backup_configs');
			notifications.success('Backup created');
		} catch (err) {
			notifications.error('Failed to create backup');
		}
	}

	// GitHub Token functions
	async function loadGithubTokenStatus() {
		try {
			hasToken = await invoke<boolean>('has_github_token');
		} catch (err) {
			console.error('Failed to check GitHub token:', err);
		}
	}

	async function saveGithubToken() {
		if (!githubToken.trim()) return;
		isSavingToken = true;
		try {
			await invoke('set_github_token', { token: githubToken });
			hasToken = true;
			githubToken = '';
			notifications.success('GitHub token saved');
		} catch (err) {
			notifications.error(`Failed to save token: ${err}`);
		} finally {
			isSavingToken = false;
		}
	}

	async function clearGithubToken() {
		isSavingToken = true;
		try {
			await invoke('clear_github_token');
			hasToken = false;
			githubToken = '';
			notifications.success('GitHub token cleared');
		} catch (err) {
			notifications.error(`Failed to clear token: ${err}`);
		} finally {
			isSavingToken = false;
		}
	}

	// MCP Server functions
	async function loadMcpServerStatus() {
		try {
			mcpServerStatus = await invoke<McpServerStatus>('get_mcp_server_status');
			mcpServerConfig = await invoke<McpServerConfig>('get_mcp_server_config');
			isSelfMcpInLibrary = await invoke<boolean>('is_self_mcp_in_library');
		} catch (err) {
			console.error('Failed to load MCP server status:', err);
		}
	}

	async function startMcpServer() {
		isServerLoading = true;
		try {
			mcpServerStatus = await invoke<McpServerStatus>('start_mcp_server');
			notifications.success('MCP server started');
		} catch (err) {
			notifications.error(`Failed to start MCP server: ${err}`);
		} finally {
			isServerLoading = false;
		}
	}

	async function stopMcpServer() {
		isServerLoading = true;
		try {
			mcpServerStatus = await invoke<McpServerStatus>('stop_mcp_server');
			notifications.success('MCP server stopped');
		} catch (err) {
			notifications.error(`Failed to stop MCP server: ${err}`);
		} finally {
			isServerLoading = false;
		}
	}

	async function updateMcpServerConfig(config: McpServerConfig) {
		try {
			await invoke('update_mcp_server_config', { config });
			mcpServerConfig = config;
			notifications.success('MCP server configuration updated');
		} catch (err) {
			notifications.error(`Failed to update config: ${err}`);
		}
	}

	async function copyConnectionConfig() {
		try {
			const config = await invoke<object>('get_mcp_server_connection_config');
			await navigator.clipboard.writeText(JSON.stringify(config, null, 2));
			notifications.success('Connection config copied to clipboard');
		} catch (err) {
			notifications.error('Failed to copy connection config');
		}
	}

	async function addSelfMcpToLibrary() {
		try {
			await invoke('add_self_mcp_to_library');
			isSelfMcpInLibrary = true;
			notifications.success('Tool Manager MCP added to library');
		} catch (err) {
			notifications.error(`Failed to add MCP: ${err}`);
		}
	}

	async function removeSelfMcpFromLibrary() {
		try {
			await invoke('remove_self_mcp_from_library');
			isSelfMcpInLibrary = false;
			notifications.success('Tool Manager MCP removed from library');
		} catch (err) {
			notifications.error(`Failed to remove MCP: ${err}`);
		}
	}

	// Gateway functions
	async function loadGatewayStatus() {
		try {
			gatewayStatus = await invoke<GatewayServerStatus>('get_gateway_status');
			gatewayConfig = await invoke<GatewayServerConfig>('get_gateway_config');
		} catch (err) {
			console.error('Failed to load Gateway status:', err);
		}
	}

	async function startGateway() {
		isGatewayLoading = true;
		try {
			gatewayStatus = await invoke<GatewayServerStatus>('start_gateway');
			notifications.success('Gateway started');
		} catch (err) {
			notifications.error(`Failed to start Gateway: ${err}`);
		} finally {
			isGatewayLoading = false;
		}
	}

	async function stopGateway() {
		isGatewayLoading = true;
		try {
			gatewayStatus = await invoke<GatewayServerStatus>('stop_gateway');
			notifications.success('Gateway stopped');
		} catch (err) {
			notifications.error(`Failed to stop Gateway: ${err}`);
		} finally {
			isGatewayLoading = false;
		}
	}

	async function updateGatewayConfig(config: GatewayServerConfig) {
		try {
			await invoke('update_gateway_config', { config });
			gatewayConfig = config;
			notifications.success('Gateway configuration updated');
		} catch (err) {
			notifications.error(`Failed to update config: ${err}`);
		}
	}

	async function copyGatewayConnectionConfig() {
		try {
			const config = await invoke<object>('get_gateway_connection_config');
			await navigator.clipboard.writeText(JSON.stringify(config, null, 2));
			notifications.success('Gateway connection config copied to clipboard');
		} catch (err) {
			notifications.error('Failed to copy connection config');
		}
	}

	async function restartBackend(mcpId: number) {
		restartingBackend = mcpId;
		try {
			await invoke<BackendInfo>('restart_gateway_backend', { mcpId });
			// Reload status to get updated backend info
			await loadGatewayStatus();
			notifications.success('Backend restarted');
		} catch (err) {
			notifications.error(`Failed to restart backend: ${err}`);
		} finally {
			restartingBackend = null;
		}
	}

	function getBackendStatusColor(status: string): string {
		switch (status) {
			case 'connected': return 'bg-green-500';
			case 'connecting': return 'bg-yellow-500';
			case 'restarting': return 'bg-yellow-500';
			case 'failed': return 'bg-red-500';
			default: return 'bg-gray-400';
		}
	}

	function getBackendStatusBadgeClass(status: string): string {
		switch (status) {
			case 'connected': return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400';
			case 'connecting': return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400';
			case 'restarting': return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400';
			case 'failed': return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400';
			default: return 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400';
		}
	}

	// Load data on mount
	$effect(() => {
		loadPaths();
		loadEditors();
		loadAppSettings();
		loadGithubTokenStatus();
		loadMcpServerStatus();
		loadGatewayStatus();
		getVersion().then(v => appVersion = v).catch(() => appVersion = '');
	});

	function viewReleaseNotes() {
		whatsNew.showCurrentReleaseNotes();
	}

	function getEditorDisplayName(editorId: string): string {
		switch (editorId) {
			case 'claude_code': return 'Claude Code';
			case 'opencode': return 'OpenCode';
			case 'codex': return 'Codex CLI';
			case 'copilot': return 'Copilot CLI';
			case 'cursor': return 'Cursor';
			case 'gemini': return 'Gemini CLI';
			default: return editorId;
		}
	}
</script>

<Header
	title="Global Settings"
	subtitle="Configure MCPs, skills, and agents across all sessions"
/>

<div class="flex-1 overflow-auto p-6 space-y-8">
	<!-- Enabled Editors -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Enabled Editors</h3>
		<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
			Enable the coding assistants you want to sync with. When enabled, skills, commands, sub-agents, and MCPs will be synced to all enabled editors simultaneously.
		</p>

		<div class="space-y-3">
			{#each editors as editor}
				<div
					class="w-full flex items-center justify-between p-4 rounded-lg border-2 transition-all {editor.isEnabled
						? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
						: 'border-gray-200 dark:border-gray-700'}"
				>
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-lg flex items-center justify-center {editor.isEnabled
							? editor.id === 'claude_code' ? 'bg-orange-500 text-white' : editor.id === 'codex' ? 'bg-lime-600 text-white' : editor.id === 'opencode' ? 'bg-emerald-500 text-white' : editor.id === 'copilot' ? 'bg-purple-500 text-white' : editor.id === 'cursor' ? 'bg-cyan-500 text-white' : editor.id === 'gemini' ? 'bg-sky-500 text-white' : 'bg-primary-500 text-white'
							: 'bg-gray-100 dark:bg-gray-800 text-gray-500'}">
							{#if editor.id === 'claude_code'}
								<span class="text-lg font-bold">C</span>
							{:else if editor.id === 'opencode'}
								<span class="text-lg font-bold">O</span>
							{:else if editor.id === 'codex'}
								<span class="text-lg font-bold">X</span>
							{:else if editor.id === 'copilot'}
								<span class="text-lg font-bold">G</span>
							{:else if editor.id === 'cursor'}
								<span class="text-lg font-bold">U</span>
							{:else if editor.id === 'gemini'}
								<span class="text-lg font-bold">M</span>
							{:else}
								<span class="text-lg font-bold">{editor.name.charAt(0)}</span>
							{/if}
						</div>
						<div class="text-left">
							<p class="font-medium text-gray-900 dark:text-white">{editor.name}</p>
							<div class="flex items-center gap-2 text-xs">
								{#if editor.isInstalled}
									<span class="flex items-center gap-1 text-green-600 dark:text-green-400">
										<Check class="w-3 h-3" />
										Installed
									</span>
								{:else}
									<span class="flex items-center gap-1 text-amber-600 dark:text-amber-400">
										<AlertCircle class="w-3 h-3" />
										Not detected
									</span>
								{/if}
							</div>
						</div>
					</div>
					<label class="relative inline-flex items-center cursor-pointer">
						<input
							type="checkbox"
							checked={editor.isEnabled}
							disabled={togglingEditor === editor.id}
							onchange={(e) => toggleEditor(editor.id, (e.target as HTMLInputElement).checked)}
							class="sr-only peer"
						/>
						<div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600 peer-disabled:opacity-50"></div>
					</label>
				</div>
			{/each}
		</div>

		{#if appSettings.enabledEditors.length === 0}
			<div class="mt-4 p-3 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg">
				<p class="text-sm text-amber-700 dark:text-amber-400">
					<AlertCircle class="w-4 h-4 inline mr-1" />
					No editors enabled. Enable at least one editor to sync your configurations.
				</p>
			</div>
		{/if}
	</div>

	<!-- GitHub Token -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
			<Key class="w-5 h-5" />
			GitHub Token
		</h3>
		<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
			Add a personal access token to increase the GitHub API rate limit from 60 to 5,000 requests per hour.
			<a
				href="https://github.com/settings/tokens"
				target="_blank"
				rel="noopener noreferrer"
				class="text-primary-600 dark:text-primary-400 hover:underline"
			>
				Create a token
			</a>
			(no scopes needed for public repos).
		</p>

		{#if hasToken}
			<div class="flex items-center gap-3 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg mb-4">
				<Check class="w-4 h-4 text-green-600 dark:text-green-400" />
				<span class="text-sm font-medium text-green-700 dark:text-green-300">Token configured</span>
			</div>
		{/if}

		<div class="flex gap-3">
			<input
				type="password"
				bind:value={githubToken}
				placeholder={hasToken ? 'Enter new token to update...' : 'ghp_xxxxxxxxxxxxxxxxxxxx'}
				class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white placeholder-gray-400 text-sm"
			/>
			<button
				onclick={saveGithubToken}
				disabled={!githubToken.trim() || isSavingToken}
				class="btn btn-primary"
			>
				{#if isSavingToken}
					<div class="w-4 h-4 mr-2 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
				{/if}
				{hasToken ? 'Update' : 'Save'}
			</button>
			{#if hasToken}
				<button
					onclick={clearGithubToken}
					disabled={isSavingToken}
					class="btn btn-secondary text-red-600 dark:text-red-400"
				>
					Clear
				</button>
			{/if}
		</div>
	</div>

	<GlobalSettings />

	<!-- MCP Server -->
	<div class="card">
		<div class="flex items-center justify-between mb-4">
			<div>
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
					<Server class="w-5 h-5" />
					MCP Server
				</h3>
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
					Expose this app as an MCP server for programmatic control
				</p>
			</div>
			<div class="flex items-center gap-3">
				{#if mcpServerStatus}
					<span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium {mcpServerStatus.isRunning ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'}">
						<span class="w-1.5 h-1.5 rounded-full {mcpServerStatus.isRunning ? 'bg-green-500' : 'bg-gray-400'}"></span>
						{mcpServerStatus.isRunning ? 'Running' : 'Stopped'}
					</span>
				{/if}
				<!-- Enable/Disable Toggle -->
				<label class="relative inline-flex items-center cursor-pointer">
					<input
						type="checkbox"
						checked={mcpServerConfig.enabled}
						onchange={(e) => {
							const enabled = (e.target as HTMLInputElement).checked;
							updateMcpServerConfig({ ...mcpServerConfig, enabled });
							if (!enabled && mcpServerStatus?.isRunning) {
								stopMcpServer();
							}
						}}
						class="sr-only peer"
					/>
					<div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
					<span class="ms-2 text-sm font-medium text-gray-700 dark:text-gray-300">
						{mcpServerConfig.enabled ? 'Enabled' : 'Disabled'}
					</span>
				</label>
			</div>
		</div>

		{#if !mcpServerConfig.enabled}
			<!-- Disabled State -->
			<div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg text-center">
				<p class="text-sm text-gray-500 dark:text-gray-400">
					MCP server is disabled. Enable it to expose this app for programmatic control.
				</p>
			</div>
		{:else if mcpServerStatus}
			<!-- Server Controls -->
			<div class="space-y-4">
				<!-- Status and URL -->
				{#if mcpServerStatus.isRunning}
					<div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm font-medium text-green-800 dark:text-green-200">Server URL</p>
								<p class="text-sm font-mono text-green-600 dark:text-green-400">{mcpServerStatus.mcpEndpoint}</p>
							</div>
							<button onclick={copyConnectionConfig} class="btn btn-secondary text-xs py-1.5 px-3">
								<Copy class="w-3 h-3 mr-1" />
								Copy Config
							</button>
						</div>
					</div>
				{/if}

				<!-- Start/Stop Buttons -->
				<div class="flex items-center gap-3">
					{#if mcpServerStatus.isRunning}
						<button
							onclick={stopMcpServer}
							disabled={isServerLoading}
							class="btn btn-secondary"
						>
							{#if isServerLoading}
								<div class="w-4 h-4 mr-2 border-2 border-gray-300 border-t-gray-600 rounded-full animate-spin"></div>
							{:else}
								<Square class="w-4 h-4 mr-2" />
							{/if}
							Stop Server
						</button>
					{:else}
						<button
							onclick={startMcpServer}
							disabled={isServerLoading}
							class="btn btn-primary"
						>
							{#if isServerLoading}
								<div class="w-4 h-4 mr-2 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
							{:else}
								<Play class="w-4 h-4 mr-2" />
							{/if}
							Start Server
						</button>
					{/if}

					<!-- Add to Library Button -->
					{#if isSelfMcpInLibrary}
						<button onclick={removeSelfMcpFromLibrary} class="btn btn-ghost text-red-600 dark:text-red-400">
							<Trash2 class="w-4 h-4 mr-2" />
							Remove from Library
						</button>
					{:else}
						<button onclick={addSelfMcpToLibrary} class="btn btn-secondary">
							<Library class="w-4 h-4 mr-2" />
							Add to Library
						</button>
					{/if}
				</div>

				<!-- Configuration -->
				<div class="pt-4 border-t border-gray-200 dark:border-gray-700">
					<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Configuration</h4>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<!-- Port -->
						<div>
							<label for="mcp-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Port</label>
							<input
								id="mcp-port"
								type="number"
								min="1024"
								max="65535"
								value={mcpServerConfig.port}
								onchange={(e) => {
									const port = parseInt((e.target as HTMLInputElement).value);
									if (port >= 1024 && port <= 65535) {
										updateMcpServerConfig({ ...mcpServerConfig, port });
									}
								}}
								disabled={mcpServerStatus.isRunning}
								class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white disabled:opacity-50 disabled:cursor-not-allowed"
							/>
							{#if mcpServerStatus.isRunning}
								<p class="text-xs text-amber-600 dark:text-amber-400 mt-1">Stop the server to change port</p>
							{/if}
						</div>

						<!-- Auto-start -->
						<div class="flex items-center">
							<label class="flex items-center cursor-pointer">
								<input
									type="checkbox"
									checked={mcpServerConfig.autoStart}
									onchange={(e) => updateMcpServerConfig({ ...mcpServerConfig, autoStart: (e.target as HTMLInputElement).checked })}
									class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
								/>
								<span class="ml-2 text-sm text-gray-700 dark:text-gray-300">Auto-start on app launch</span>
							</label>
						</div>
					</div>
				</div>

				<!-- Available Tools Info -->
				<div class="pt-4 border-t border-gray-200 dark:border-gray-700">
					<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Available Tools</h4>
					<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
						The MCP server exposes tools for managing:
					</p>
					<div class="flex flex-wrap gap-2">
						<span class="px-2 py-1 bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300 rounded text-xs">MCPs</span>
						<span class="px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded text-xs">Projects</span>
						<span class="px-2 py-1 bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded text-xs">Skills</span>
						<span class="px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded text-xs">Sub-Agents</span>
						<span class="px-2 py-1 bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 rounded text-xs">Hooks</span>
					</div>
				</div>
			</div>
		{:else}
			<div class="flex items-center justify-center py-8">
				<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600"></div>
			</div>
		{/if}
	</div>

	<!-- MCP Gateway -->
	<div class="card">
		<div class="flex items-center justify-between mb-4">
			<div>
				<h3 class="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
					<Network class="w-5 h-5" />
					MCP Gateway
				</h3>
				<p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
					Aggregate multiple MCPs into a single endpoint for Claude
				</p>
			</div>
			<div class="flex items-center gap-3">
				{#if gatewayStatus}
					<span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium {gatewayStatus.isRunning ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'}">
						<span class="w-1.5 h-1.5 rounded-full {gatewayStatus.isRunning ? 'bg-green-500' : 'bg-gray-400'}"></span>
						{gatewayStatus.isRunning ? `Running (${gatewayStatus.totalTools} tools)` : 'Stopped'}
					</span>
				{/if}
				<!-- Enable/Disable Toggle -->
				<label class="relative inline-flex items-center cursor-pointer">
					<input
						type="checkbox"
						checked={gatewayConfig.enabled}
						onchange={(e) => {
							const enabled = (e.target as HTMLInputElement).checked;
							updateGatewayConfig({ ...gatewayConfig, enabled });
							if (!enabled && gatewayStatus?.isRunning) {
								stopGateway();
							}
						}}
						class="sr-only peer"
					/>
					<div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 dark:peer-focus:ring-primary-800 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-primary-600"></div>
					<span class="ms-2 text-sm font-medium text-gray-700 dark:text-gray-300">
						{gatewayConfig.enabled ? 'Enabled' : 'Disabled'}
					</span>
				</label>
			</div>
		</div>

		{#if !gatewayConfig.enabled}
			<!-- Disabled State -->
			<div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-lg text-center">
				<p class="text-sm text-gray-500 dark:text-gray-400">
					Gateway is disabled. Enable it to aggregate MCPs into a single endpoint.
				</p>
			</div>
		{:else if gatewayStatus}
			<!-- Gateway Controls -->
			<div class="space-y-4">
				<!-- Status and URL -->
				{#if gatewayStatus.isRunning}
					<div class="p-4 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
						<div class="flex items-center justify-between">
							<div>
								<p class="text-sm font-medium text-green-800 dark:text-green-200">Gateway URL</p>
								<p class="text-sm font-mono text-green-600 dark:text-green-400">{gatewayStatus.mcpEndpoint}</p>
							</div>
							<button onclick={copyGatewayConnectionConfig} class="btn btn-secondary text-xs py-1.5 px-3">
								<Copy class="w-3 h-3 mr-1" />
								Copy Config
							</button>
						</div>
					</div>
				{/if}

				<!-- Start/Stop Buttons -->
				<div class="flex items-center gap-3">
					{#if gatewayStatus.isRunning}
						<button
							onclick={stopGateway}
							disabled={isGatewayLoading}
							class="btn btn-secondary"
						>
							{#if isGatewayLoading}
								<div class="w-4 h-4 mr-2 border-2 border-gray-300 border-t-gray-600 rounded-full animate-spin"></div>
							{:else}
								<Square class="w-4 h-4 mr-2" />
							{/if}
							Stop Gateway
						</button>
					{:else}
						<button
							onclick={startGateway}
							disabled={isGatewayLoading}
							class="btn btn-primary"
						>
							{#if isGatewayLoading}
								<div class="w-4 h-4 mr-2 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
							{:else}
								<Play class="w-4 h-4 mr-2" />
							{/if}
							Start Gateway
						</button>
					{/if}
				</div>

				<!-- Configuration -->
				<div class="pt-4 border-t border-gray-200 dark:border-gray-700">
					<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Configuration</h4>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<!-- Port -->
						<div>
							<label for="gateway-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Port</label>
							<input
								id="gateway-port"
								type="number"
								min="1024"
								max="65535"
								value={gatewayConfig.port}
								onchange={(e) => {
									const port = parseInt((e.target as HTMLInputElement).value);
									if (port >= 1024 && port <= 65535) {
										updateGatewayConfig({ ...gatewayConfig, port });
									}
								}}
								disabled={gatewayStatus.isRunning}
								class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white disabled:opacity-50 disabled:cursor-not-allowed"
							/>
							{#if gatewayStatus.isRunning}
								<p class="text-xs text-amber-600 dark:text-amber-400 mt-1">Stop the gateway to change port</p>
							{/if}
						</div>

						<!-- Auto-start -->
						<div class="flex items-center">
							<label class="flex items-center cursor-pointer">
								<input
									type="checkbox"
									checked={gatewayConfig.autoStart}
									onchange={(e) => updateGatewayConfig({ ...gatewayConfig, autoStart: (e.target as HTMLInputElement).checked })}
									class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600"
								/>
								<span class="ml-2 text-sm text-gray-700 dark:text-gray-300">Auto-start on app launch</span>
							</label>
						</div>
					</div>
				</div>

				<!-- Connected Backends -->
				{#if gatewayStatus.isRunning && gatewayStatus.connectedBackends.length > 0}
					<div class="pt-4 border-t border-gray-200 dark:border-gray-700">
						<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Connected MCPs ({gatewayStatus.connectedBackends.length})</h4>
						<div class="space-y-2">
							{#each gatewayStatus.connectedBackends as backend}
								<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
									<div class="flex items-center gap-3">
										<span class="w-2 h-2 rounded-full {getBackendStatusColor(backend.status)}"></span>
										<div>
											<p class="text-sm font-medium text-gray-900 dark:text-white">{backend.mcpName}</p>
											<p class="text-xs text-gray-500 dark:text-gray-400">
												{backend.toolCount} tools
												{#if backend.errorMessage}
													<span class="text-red-500"> - {backend.errorMessage}</span>
												{/if}
											</p>
										</div>
									</div>
									<div class="flex items-center gap-2">
										<span class="px-2 py-0.5 rounded text-xs font-medium {getBackendStatusBadgeClass(backend.status)}">
											{backend.status}
										</span>
										<button
											onclick={() => restartBackend(backend.mcpId)}
											disabled={restartingBackend === backend.mcpId}
											class="p-1.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
											title="Restart backend"
										>
											{#if restartingBackend === backend.mcpId}
												<div class="w-4 h-4 border-2 border-gray-300 border-t-gray-600 rounded-full animate-spin"></div>
											{:else}
												<RotateCw class="w-4 h-4" />
											{/if}
										</button>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{:else if gatewayStatus.isRunning}
					<div class="pt-4 border-t border-gray-200 dark:border-gray-700">
						<p class="text-sm text-gray-500 dark:text-gray-400">
							No MCPs connected. Add MCPs to the gateway from the Library page.
						</p>
					</div>
				{/if}

				<!-- Info -->
				<div class="pt-4 border-t border-gray-200 dark:border-gray-700">
					<p class="text-xs text-gray-500 dark:text-gray-400">
						The gateway aggregates tools from multiple MCPs into a single endpoint. Add MCPs to the gateway
						from the Library page, then use the gateway URL in your Claude config instead of individual MCPs.
						Tool names are prefixed with their source MCP name (e.g., <code class="bg-gray-100 dark:bg-gray-800 px-1 rounded">filesystem__read_file</code>).
					</p>
				</div>
			</div>
		{:else}
			<div class="flex items-center justify-center py-8">
				<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600"></div>
			</div>
		{/if}
	</div>

	<!-- Configuration Paths -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Configuration Paths</h3>

		<!-- Claude Code Paths -->
		{#if claudePaths}
			<div class="mb-6">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-orange-500 flex items-center justify-center text-white text-xs font-bold">C</div>
					Claude Code
				</h4>
				<div class="space-y-2 ml-7">
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FolderOpen class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Config Directory</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{claudePaths.claudeDir}</p>
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FileText class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Main Config</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{claudePaths.claudeJson}</p>
							</div>
						</div>
						<button
							onclick={() => openConfigFile(claudePaths!.claudeJson)}
							class="btn btn-ghost text-xs py-1 px-2"
						>
							Open
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- OpenCode Paths -->
		{#if opencodePaths}
			<div class="mb-6">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-emerald-500 flex items-center justify-center text-white text-xs font-bold">O</div>
					OpenCode
				</h4>
				<div class="space-y-2 ml-7">
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FolderOpen class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Config Directory</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{opencodePaths.configDir}</p>
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FileText class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Main Config</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{opencodePaths.configFile}</p>
							</div>
						</div>
						<button
							onclick={() => openConfigFile(opencodePaths!.configFile)}
							class="btn btn-ghost text-xs py-1 px-2"
						>
							Open
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Codex CLI Paths -->
		{#if codexPaths}
			<div class="mb-6">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-lime-600 flex items-center justify-center text-white text-xs font-bold">X</div>
					Codex CLI
				</h4>
				<div class="space-y-2 ml-7">
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FolderOpen class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Config Directory</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{codexPaths.configDir}</p>
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FileText class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Main Config</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{codexPaths.configFile}</p>
							</div>
						</div>
						<button
							onclick={() => openConfigFile(codexPaths!.configFile)}
							class="btn btn-ghost text-xs py-1 px-2"
						>
							Open
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Copilot CLI Paths -->
		{#if copilotPaths}
			<div class="mb-6">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-purple-500 flex items-center justify-center text-white text-xs font-bold">G</div>
					Copilot CLI
				</h4>
				<div class="space-y-2 ml-7">
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FolderOpen class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Config Directory</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{copilotPaths.configDir}</p>
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FileText class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">MCP Config</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{copilotPaths.mcpConfigFile}</p>
							</div>
						</div>
						<button
							onclick={() => openConfigFile(copilotPaths!.mcpConfigFile)}
							class="btn btn-ghost text-xs py-1 px-2"
						>
							Open
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Cursor Paths -->
		{#if cursorPaths}
			<div class="mb-6">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-cyan-500 flex items-center justify-center text-white text-xs font-bold">U</div>
					Cursor
				</h4>
				<div class="space-y-2 ml-7">
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FolderOpen class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Config Directory</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{cursorPaths.configDir}</p>
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FileText class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">MCP Config</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{cursorPaths.mcpConfigFile}</p>
							</div>
						</div>
						<button
							onclick={() => openConfigFile(cursorPaths!.mcpConfigFile)}
							class="btn btn-ghost text-xs py-1 px-2"
						>
							Open
						</button>
					</div>
				</div>
			</div>
		{/if}

		<!-- Gemini CLI Paths -->
		{#if geminiPaths}
			<div>
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-sky-500 flex items-center justify-center text-white text-xs font-bold">M</div>
					Gemini CLI
				</h4>
				<div class="space-y-2 ml-7">
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FolderOpen class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Config Directory</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{geminiPaths.configDir}</p>
							</div>
						</div>
					</div>
					<div class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
						<div class="flex items-center gap-2">
							<FileText class="w-4 h-4 text-gray-400" />
							<div>
								<p class="text-xs font-medium text-gray-700 dark:text-gray-300">Settings File</p>
								<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{geminiPaths.settingsFile}</p>
							</div>
						</div>
						<button
							onclick={() => openConfigFile(geminiPaths!.settingsFile)}
							class="btn btn-ghost text-xs py-1 px-2"
						>
							Open
						</button>
					</div>
				</div>
			</div>
		{/if}

		{#if !claudePaths && !opencodePaths && !codexPaths && !copilotPaths && !cursorPaths && !geminiPaths}
			<div class="flex items-center justify-center py-8">
				<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600"></div>
			</div>
		{/if}
	</div>

	<!-- Backup -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Backup & Restore</h3>
		<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
			Create a backup of your MCP configurations before making changes.
		</p>
		<button onclick={backupConfigs} class="btn btn-secondary">
			<RefreshCw class="w-4 h-4 mr-2" />
			Create Backup
		</button>
	</div>

	<!-- About -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">About</h3>
		<div class="flex items-center justify-between">
			<div>
				<p class="text-sm font-medium text-gray-900 dark:text-white">
					Claude Code Tool Manager
				</p>
				{#if appVersion}
					<p class="text-sm text-gray-500 dark:text-gray-400">
						Version {appVersion}
					</p>
				{/if}
			</div>
			<button onclick={viewReleaseNotes} class="btn btn-secondary">
				<Sparkles class="w-4 h-4 mr-2" />
				What's New
			</button>
		</div>
	</div>
</div>
