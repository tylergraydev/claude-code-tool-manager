<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { notifications } from '$lib/stores';
	import { Save, Eye, EyeOff } from 'lucide-svelte';

	type ClaudeSettings = {
		authMode: string;
		apiKey: string | null;
		autoMountClaudeDir: boolean;
		autoInstall: boolean;
	};

	let settings = $state<ClaudeSettings>({
		authMode: 'max',
		apiKey: null,
		autoMountClaudeDir: true,
		autoInstall: false,
	});

	let isLoading = $state(true);
	let isSaving = $state(false);
	let showApiKey = $state(false);

	onMount(async () => {
		try {
			settings = await invoke<ClaudeSettings>('get_container_claude_settings');
		} catch (e) {
			console.error('Failed to load container Claude settings:', e);
		} finally {
			isLoading = false;
		}
	});

	async function save() {
		isSaving = true;
		try {
			await invoke('set_container_claude_settings', { settings });
			notifications.success('Container Claude settings saved');
		} catch (e) {
			notifications.error(`Failed to save: ${e}`);
		} finally {
			isSaving = false;
		}
	}
</script>

<div class="p-6 space-y-6 max-w-2xl">
	{#if isLoading}
		<div class="flex items-center justify-center py-8">
			<div class="animate-spin w-6 h-6 border-2 border-primary-500 border-t-transparent rounded-full"></div>
		</div>
	{:else}
		<div>
			<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Claude Code in Containers</h3>
			<p class="text-sm text-gray-500 dark:text-gray-400">Configure how Claude Code authenticates and installs in your dev containers.</p>
		</div>

		<!-- Auth Mode -->
		<div class="space-y-3">
			<label class="block text-sm font-medium text-gray-700 dark:text-gray-300">Authentication Method</label>
			<div class="space-y-2">
				<label class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors {settings.authMode === 'max' ? 'border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}">
					<input type="radio" bind:group={settings.authMode} value="max" class="mt-1" />
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">Max Plan (OAuth)</p>
						<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Mounts your ~/.claude directory into containers. No API key needed — uses your existing login session.</p>
					</div>
				</label>
				<label class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors {settings.authMode === 'api_key' ? 'border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}">
					<input type="radio" bind:group={settings.authMode} value="api_key" class="mt-1" />
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">API Key</p>
						<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Injects ANTHROPIC_API_KEY as an environment variable in containers.</p>
					</div>
				</label>
			</div>
		</div>

		<!-- API Key input (only for api_key mode) -->
		{#if settings.authMode === 'api_key'}
			<div>
				<label for="api-key" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">API Key</label>
				<div class="relative">
					<input
						id="api-key"
						type={showApiKey ? 'text' : 'password'}
						bind:value={settings.apiKey}
						placeholder="sk-ant-..."
						class="input w-full pr-10 font-mono text-sm"
					/>
					<button
						type="button"
						onclick={() => showApiKey = !showApiKey}
						class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost p-1"
						aria-label={showApiKey ? 'Hide API key' : 'Show API key'}
					>
						{#if showApiKey}
							<EyeOff class="w-4 h-4" />
						{:else}
							<Eye class="w-4 h-4" />
						{/if}
					</button>
				</div>
			</div>
		{/if}

		<!-- Auto-mount toggle (only for max mode) -->
		{#if settings.authMode === 'max'}
			<label class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700">
				<div>
					<p class="text-sm font-medium text-gray-900 dark:text-white">Auto-mount ~/.claude directory</p>
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Shares your OAuth session with containers so you don't need to re-login</p>
				</div>
				<input type="checkbox" bind:checked={settings.autoMountClaudeDir} class="rounded" />
			</label>
		{/if}

		<!-- Auto-install toggle -->
		<label class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700">
			<div>
				<p class="text-sm font-medium text-gray-900 dark:text-white">Auto-install Claude Code</p>
				<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Installs Claude Code globally (npm i -g @anthropic-ai/claude-code) when a container first starts</p>
			</div>
			<input type="checkbox" bind:checked={settings.autoInstall} class="rounded" />
		</label>

		<!-- Save -->
		<div class="flex justify-end pt-2">
			<button onclick={save} disabled={isSaving} class="btn btn-primary">
				<Save class="w-4 h-4 mr-2" />
				{isSaving ? 'Saving...' : 'Save Settings'}
			</button>
		</div>
	{/if}
</div>
