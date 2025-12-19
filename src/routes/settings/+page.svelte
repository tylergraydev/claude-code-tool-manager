<script lang="ts">
	import { Header } from '$lib/components/layout';
	import { GlobalSettings } from '$lib/components/global';
	import { invoke } from '@tauri-apps/api/core';
	import { notifications } from '$lib/stores';
	import { FolderOpen, FileText, RefreshCw } from 'lucide-svelte';

	interface ClaudePaths {
		claudeDir: string;
		globalSettings: string;
		pluginsDir: string;
	}

	let claudePaths = $state<ClaudePaths | null>(null);

	async function loadPaths() {
		try {
			claudePaths = await invoke<ClaudePaths>('get_claude_paths');
		} catch (err) {
			console.error('Failed to load paths:', err);
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

	// Load paths on mount
	$effect(() => {
		loadPaths();
	});
</script>

<Header
	title="Global Settings"
	subtitle="Configure MCPs available across all Claude Code sessions"
/>

<div class="flex-1 overflow-auto p-6 space-y-8">
	<GlobalSettings />

	<!-- Claude Paths Info -->
	<div class="card">
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Configuration Paths</h3>

		{#if claudePaths}
			<div class="space-y-3">
				<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
					<div class="flex items-center gap-3">
						<FolderOpen class="w-5 h-5 text-gray-400" />
						<div>
							<p class="text-sm font-medium text-gray-900 dark:text-white">Claude Directory</p>
							<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{claudePaths.claudeDir}</p>
						</div>
					</div>
				</div>

				<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
					<div class="flex items-center gap-3">
						<FileText class="w-5 h-5 text-gray-400" />
						<div>
							<p class="text-sm font-medium text-gray-900 dark:text-white">Global Settings</p>
							<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{claudePaths.globalSettings}</p>
						</div>
					</div>
					<button
						onclick={() => openConfigFile(claudePaths!.globalSettings)}
						class="btn btn-ghost text-xs"
					>
						Open
					</button>
				</div>

				<div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg">
					<div class="flex items-center gap-3">
						<FolderOpen class="w-5 h-5 text-gray-400" />
						<div>
							<p class="text-sm font-medium text-gray-900 dark:text-white">Plugins Directory</p>
							<p class="text-xs text-gray-500 dark:text-gray-400 font-mono">{claudePaths.pluginsDir}</p>
						</div>
					</div>
				</div>
			</div>
		{:else}
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
</div>
