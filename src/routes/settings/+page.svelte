<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { GlobalSettings } from '$lib/components/global';
	import { invoke } from '@tauri-apps/api/core';
	import { notifications, whatsNew } from '$lib/stores';
	import { FolderOpen, FileText, RefreshCw, Sparkles, Check, AlertCircle } from 'lucide-svelte';
	import { getVersion } from '@tauri-apps/api/app';

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

	interface EditorInfo {
		id: string;
		name: string;
		isInstalled: boolean;
		configPath: string;
	}

	interface AppSettings {
		defaultEditor: string;
	}

	let claudePaths = $state<ClaudePaths | null>(null);
	let opencodePaths = $state<OpenCodePaths | null>(null);
	let editors = $state<EditorInfo[]>([]);
	let appSettings = $state<AppSettings>({ defaultEditor: 'claude_code' });
	let savingSettings = $state(false);

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

	async function updateDefaultEditor(editorId: string) {
		savingSettings = true;
		try {
			await invoke('update_app_settings', { settings: { defaultEditor: editorId } });
			appSettings.defaultEditor = editorId;
			notifications.success('Default editor updated');
		} catch (err) {
			notifications.error('Failed to update default editor');
		} finally {
			savingSettings = false;
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

	// Load data on mount
	$effect(() => {
		loadPaths();
		loadEditors();
		loadAppSettings();
		getVersion().then(v => appVersion = v).catch(() => appVersion = '');
	});

	function viewReleaseNotes() {
		whatsNew.showCurrentReleaseNotes();
	}

	function getEditorDisplayName(editorId: string): string {
		return editorId === 'claude_code' ? 'Claude Code' : 'OpenCode';
	}
</script>

<Header
	title="Global Settings"
	subtitle="Configure MCPs, skills, and agents across all sessions"
/>

<div class="flex-1 overflow-auto p-6 space-y-8">
	<!-- Default Editor Selection -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Default Editor</h3>
		<p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
			Choose which coding assistant to use by default for new projects. You can override this per project.
		</p>

		<div class="space-y-3">
			{#each editors as editor}
				<button
					onclick={() => updateDefaultEditor(editor.id)}
					disabled={savingSettings}
					class="w-full flex items-center justify-between p-4 rounded-lg border-2 transition-all {appSettings.defaultEditor === editor.id
						? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
						: 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
				>
					<div class="flex items-center gap-3">
						<div class="w-10 h-10 rounded-lg flex items-center justify-center {appSettings.defaultEditor === editor.id
							? 'bg-primary-500 text-white'
							: 'bg-gray-100 dark:bg-gray-800 text-gray-500'}">
							{#if editor.id === 'claude_code'}
								<span class="text-lg font-bold">C</span>
							{:else}
								<span class="text-lg font-bold">O</span>
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
					{#if appSettings.defaultEditor === editor.id}
						<div class="w-5 h-5 rounded-full bg-primary-500 flex items-center justify-center">
							<Check class="w-3 h-3 text-white" />
						</div>
					{/if}
				</button>
			{/each}
		</div>
	</div>

	<GlobalSettings />

	<!-- Configuration Paths -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Configuration Paths</h3>

		<!-- Claude Code Paths -->
		{#if claudePaths}
			<div class="mb-6">
				<h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 flex items-center gap-2">
					<div class="w-5 h-5 rounded bg-primary-500 flex items-center justify-center text-white text-xs font-bold">C</div>
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
			<div>
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

		{#if !claudePaths && !opencodePaths}
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
