<script lang="ts">
	import { onMount } from 'svelte';
	import { Header } from '$lib/components/layout';
	import { EnvVarsEditor } from '$lib/components/env-vars';
	import { claudeSettingsLibrary, projectsStore, notifications } from '$lib/stores';
	import type { ClaudeSettings, ClaudeSettingsScope } from '$lib/types';
	import { CLAUDE_SETTINGS_SCOPE_LABELS } from '$lib/types';
	import { RefreshCw, FolderOpen, User, FileText } from 'lucide-svelte';

	onMount(async () => {
		await projectsStore.loadProjects();
		await claudeSettingsLibrary.load();
	});

	function handleProjectChange(e: Event) {
		const target = e.target as HTMLSelectElement;
		const value = target.value;
		claudeSettingsLibrary.setProjectPath(value || null);
		claudeSettingsLibrary.load();
	}

	async function handleRefresh() {
		await claudeSettingsLibrary.load();
		notifications.success('Settings refreshed');
	}

	async function handleSave(settings: ClaudeSettings) {
		try {
			await claudeSettingsLibrary.save(settings);
			notifications.success('Environment variables saved');
		} catch (err) {
			notifications.error('Failed to save environment variables');
		}
	}

	const scopes: { key: ClaudeSettingsScope; icon: typeof User }[] = [
		{ key: 'user', icon: User },
		{ key: 'project', icon: FolderOpen },
		{ key: 'local', icon: FileText }
	];

	function getSettingCount(scope: ClaudeSettingsScope): number {
		if (!claudeSettingsLibrary.settings) return 0;
		const s =
			scope === 'user'
				? claudeSettingsLibrary.settings.user
				: scope === 'project'
					? claudeSettingsLibrary.settings.project
					: claudeSettingsLibrary.settings.local;
		if (!s) return 0;
		return Object.keys(s.env ?? {}).length;
	}
</script>

<Header
	title="Env Variables"
	subtitle="Configure environment variables for Claude Code runtime"
/>

<div class="flex-1 overflow-auto p-6">
	<div class="flex flex-wrap items-center gap-4 mb-6">
		<div class="flex items-center gap-2">
			<FolderOpen class="w-4 h-4 text-gray-500 dark:text-gray-400" />
			<select
				value={claudeSettingsLibrary.projectPath ?? ''}
				onchange={handleProjectChange}
				class="input text-sm"
			>
				<option value="">No project</option>
				{#each projectsStore.projects as project}
					<option value={project.path}>{project.name}</option>
				{/each}
			</select>
		</div>

		<div class="flex-1 min-w-[300px]">
			<div class="flex gap-1 bg-gray-100 dark:bg-gray-700/50 rounded-lg p-1">
				{#each scopes as { key, icon }}
					{@const isDisabled = key !== 'user' && !claudeSettingsLibrary.projectPath}
					{@const isActive = claudeSettingsLibrary.selectedScope === key}
					{@const count = getSettingCount(key)}
					<button
						onclick={() => claudeSettingsLibrary.setScope(key)}
						disabled={isDisabled}
						class="flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors flex-1
							{isActive
							? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
							: isDisabled
								? 'text-gray-400 dark:text-gray-500 cursor-not-allowed'
								: 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
						title={CLAUDE_SETTINGS_SCOPE_LABELS[key].description}
					>
						<svelte:component this={icon} class="w-4 h-4" />
						{CLAUDE_SETTINGS_SCOPE_LABELS[key].label}
						{#if count > 0}
							<span
								class="ml-1 px-1.5 py-0.5 text-xs rounded-full
									{isActive
								? 'bg-primary-100 dark:bg-primary-900/50 text-primary-700 dark:text-primary-300'
								: 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300'}"
							>
								{count}
							</span>
						{/if}
					</button>
				{/each}
			</div>
		</div>

		<div class="flex items-center gap-2">
			<button
				onclick={handleRefresh}
				class="btn btn-ghost"
				title="Refresh from settings files"
			>
				<RefreshCw class="w-4 h-4" />
			</button>
		</div>
	</div>

	{#if claudeSettingsLibrary.isLoading}
		<div class="flex items-center justify-center py-20">
			<div
				class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"
			></div>
		</div>
	{:else if claudeSettingsLibrary.error}
		<div
			class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400"
		>
			{claudeSettingsLibrary.error}
		</div>
	{:else if claudeSettingsLibrary.currentScopeSettings}
		<EnvVarsEditor
			settings={claudeSettingsLibrary.currentScopeSettings}
			onsave={handleSave}
		/>
	{:else}
		<div class="text-center py-20 text-gray-400 dark:text-gray-500">
			<p>Select a scope to view settings</p>
		</div>
	{/if}
</div>
