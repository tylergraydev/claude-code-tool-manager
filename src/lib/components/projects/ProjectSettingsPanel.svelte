<script lang="ts">
	import type { Project } from '$lib/types';
	import type { ClaudeSettings, ClaudeSettingsScope } from '$lib/types';
	import { CLAUDE_SETTINGS_SCOPE_LABELS } from '$lib/types';
	import { claudeSettingsLibrary, notifications } from '$lib/stores';
	import { SETTINGS_CATEGORIES } from '$lib/components/settings';
	import { FolderOpen, FileText, RefreshCw, Info } from 'lucide-svelte';

	import { ModelConfigEditor, AttributionEditor } from '$lib/components/claude-settings';
	import { SandboxConfigEditor } from '$lib/components/sandbox';
	import { PluginListEditor, MarketplaceEditor } from '$lib/components/plugins';
	import { EnvVarsEditor } from '$lib/components/env-vars';
	import { UITogglesEditor } from '$lib/components/ui-toggles';
	import { FileSuggestionEditor } from '$lib/components/file-suggestion';
	import { SessionCleanupEditor } from '$lib/components/session-cleanup';
	import { AuthHelpersEditor } from '$lib/components/auth-helpers';
	import { McpApprovalEditor } from '$lib/components/mcp-approval';

	type Props = {
		project: Project;
		activeSection?: string;
		onSectionChange?: (section: string) => void;
	};

	let { project, activeSection = 'models', onSectionChange }: Props = $props();

	const scopedCategories = SETTINGS_CATEGORIES.filter(c => c.type === 'scoped');

	const scopes: { key: ClaudeSettingsScope; icon: typeof FolderOpen; label: string; description: string }[] = [
		{ key: 'project', icon: FolderOpen, label: CLAUDE_SETTINGS_SCOPE_LABELS['project'].label, description: CLAUDE_SETTINGS_SCOPE_LABELS['project'].description },
		{ key: 'local', icon: FileText, label: CLAUDE_SETTINGS_SCOPE_LABELS['local'].label, description: CLAUDE_SETTINGS_SCOPE_LABELS['local'].description }
	];

	function handleSectionChange(sectionId: string) {
		onSectionChange?.(sectionId);
	}

	async function handleRefresh() {
		await claudeSettingsLibrary.load();
		notifications.success('Settings refreshed');
	}

	async function save(settings: ClaudeSettings, successMsg: string, errorMsg: string) {
		try {
			await claudeSettingsLibrary.save(settings);
			notifications.success(successMsg);
		} catch (err) {
			notifications.error(errorMsg);
		}
	}
</script>

<div class="flex-1 overflow-hidden flex flex-col">
	<!-- Scope toggle + refresh bar -->
	<div class="flex items-center gap-4 px-6 py-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/30">
		<div class="flex gap-1 bg-gray-100 dark:bg-gray-700/50 rounded-lg p-1">
			{#each scopes as { key, icon, label, description }}
				{@const isActive = claudeSettingsLibrary.selectedScope === key}
				<button
					onclick={() => claudeSettingsLibrary.setScope(key)}
					class="flex items-center gap-2 px-4 py-1.5 rounded-md text-sm font-medium transition-colors
						{isActive
						? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm'
						: 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-white'}"
					title={description}
				>
					<svelte:component this={icon} class="w-4 h-4" />
					{label}
				</button>
			{/each}
		</div>
		<div class="flex-1"></div>
		<button
			onclick={handleRefresh}
			class="btn btn-ghost"
			title="Refresh from settings files"
		>
			<RefreshCw class="w-4 h-4" />
		</button>
	</div>

	<!-- Settings content: left nav + right editor -->
	<div class="flex-1 overflow-hidden flex">
		<!-- Left sub-nav -->
		<nav class="w-44 border-r border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 overflow-y-auto flex-shrink-0">
			<div class="p-3">
				{#each scopedCategories as category}
					{@const isActive = activeSection === category.id}
					<button
						onclick={() => handleSectionChange(category.id)}
						class="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm font-medium transition-colors mb-0.5
							{isActive
								? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
								: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50'}"
					>
						<svelte:component this={category.icon} class="w-4 h-4 flex-shrink-0" />
						{category.label}
					</button>
				{/each}
			</div>
		</nav>

		<!-- Right content area -->
		<div class="flex-1 overflow-auto p-6">
			{#if claudeSettingsLibrary.isLoading}
				<div class="flex items-center justify-center py-20">
					<div class="animate-spin w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full"></div>
				</div>
			{:else if claudeSettingsLibrary.error}
				<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 text-red-700 dark:text-red-400">
					{claudeSettingsLibrary.error}
				</div>
			{:else if claudeSettingsLibrary.currentScopeSettings}
				{@const settings = claudeSettingsLibrary.currentScopeSettings}

				{#if !project.hasSettingsFile && claudeSettingsLibrary.selectedScope === 'project'}
					<div class="flex items-start gap-3 mb-6 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
						<Info class="w-5 h-5 text-blue-500 flex-shrink-0 mt-0.5" />
						<p class="text-sm text-blue-700 dark:text-blue-300">
							No project settings file exists yet. Saving will create <code class="bg-blue-100 dark:bg-blue-800/50 px-1 rounded">.claude/settings.json</code> in the project folder.
						</p>
					</div>
				{/if}

				{#if activeSection === 'models'}
					<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
						<ModelConfigEditor
							{settings}
							onsave={(s) => save(s, 'Model settings saved', 'Failed to save model settings')}
						/>
						<AttributionEditor
							{settings}
							onsave={(s) => save(s, 'Attribution settings saved', 'Failed to save attribution settings')}
						/>
					</div>
				{:else if activeSection === 'security'}
					<SandboxConfigEditor
						{settings}
						onsave={(s) => save(s, 'Security settings saved', 'Failed to save security settings')}
					/>
				{:else if activeSection === 'plugins'}
					<div class="space-y-6">
						<PluginListEditor
							{settings}
							onsave={(s) => save(s, 'Plugin settings saved', 'Failed to save plugin settings')}
						/>
						<MarketplaceEditor
							{settings}
							onsave={(s) => save(s, 'Marketplace settings saved', 'Failed to save marketplace settings')}
						/>
					</div>
				{:else if activeSection === 'environment'}
					<EnvVarsEditor
						{settings}
						onsave={(s) => save(s, 'Environment variables saved', 'Failed to save environment variables')}
					/>
				{:else if activeSection === 'interface'}
					<UITogglesEditor
						{settings}
						onsave={(s) => save(s, 'Interface settings saved', 'Failed to save interface settings')}
					/>
				{:else if activeSection === 'files'}
					<FileSuggestionEditor
						{settings}
						onsave={(s) => save(s, 'File settings saved', 'Failed to save file settings')}
					/>
				{:else if activeSection === 'session'}
					<SessionCleanupEditor
						{settings}
						onsave={(s) => save(s, 'Session settings saved', 'Failed to save session settings')}
					/>
				{:else if activeSection === 'authentication'}
					<AuthHelpersEditor
						{settings}
						onsave={(s) => save(s, 'Auth settings saved', 'Failed to save auth settings')}
					/>
				{:else if activeSection === 'mcp-approval'}
					<McpApprovalEditor
						{settings}
						onsave={(s) => save(s, 'MCP approval settings saved', 'Failed to save MCP approval settings')}
					/>
				{/if}
			{:else}
				<div class="text-center py-20 text-gray-400 dark:text-gray-500">
					<p>No settings available for this scope</p>
				</div>
			{/if}
		</div>
	</div>
</div>
