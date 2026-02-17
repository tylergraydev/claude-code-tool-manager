<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Header } from '$lib/components/layout';
	import { SETTINGS_CATEGORIES } from '$lib/components/settings';
	import {
		SettingsModelsTab,
		SettingsSecurityTab,
		SettingsPluginsTab,
		SettingsEnvironmentTab,
		SettingsInterfaceTab,
		SettingsFilesTab,
		SettingsSessionTab,
		SettingsAuthTab,
		SettingsMcpApprovalTab,
		SettingsKeybindingsTab,
		SettingsSpinnerVerbsTab,
		SettingsAdminTab,
		SettingsEditorSyncTab
	} from '$lib/components/settings/tabs';

	const scopedCategories = SETTINGS_CATEGORIES.filter(c => c.type === 'scoped');
	const standaloneCategories = SETTINGS_CATEGORIES.filter(c => c.type === 'standalone');

	const activeTab = $derived($page.url.searchParams.get('tab') ?? 'models');
	const activeCategory = $derived(SETTINGS_CATEGORIES.find(c => c.id === activeTab) ?? SETTINGS_CATEGORIES[0]);

	function switchTab(tabId: string) {
		goto(`/settings?tab=${tabId}`, { replaceState: true });
	}

	const TAB_SUBTITLES: Record<string, string> = {
		'models': 'Configure model defaults, output behavior, and git attribution',
		'security': 'Configure bash command isolation, network access, and security settings',
		'plugins': 'Manage enabled plugins and custom marketplace sources',
		'environment': 'Configure environment variables for Claude Code runtime',
		'interface': 'Control visual and interaction preferences for Claude Code',
		'files': 'Configure custom @ file autocomplete suggestions',
		'session': 'Configure session cleanup, updates, teammate mode, and plans',
		'authentication': 'Configure scripts that provide authentication credentials and API keys',
		'mcp-approval': 'Control which project-level MCP servers are automatically approved',
		'keybindings': 'Customize keyboard shortcuts for Claude Code',
		'spinner-verbs': 'Customize the action verbs shown in Claude Code\'s spinner',
		'admin': 'View enterprise managed settings deployed by your IT administrator',
		'editor-sync': 'Configure editors, servers, tokens, paths, and backups'
	};
</script>

<Header
	title="Settings"
	subtitle={TAB_SUBTITLES[activeTab] ?? 'Configure Claude Code settings'}
/>

<div class="flex-1 overflow-hidden flex">
	<!-- Left Nav -->
	<nav class="w-52 border-r border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 overflow-y-auto flex-shrink-0">
		<div class="p-3">
			<p class="text-[10px] font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500 px-3 mb-2">
				Configuration
			</p>
			{#each scopedCategories as category}
				{@const isActive = activeTab === category.id}
				<button
					onclick={() => switchTab(category.id)}
					class="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm font-medium transition-colors mb-0.5
						{isActive
							? 'bg-primary-50 text-primary-700 dark:bg-primary-900/50 dark:text-primary-300'
							: 'text-gray-600 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50'}"
				>
					<svelte:component this={category.icon} class="w-4 h-4 flex-shrink-0" />
					{category.label}
				</button>
			{/each}

			<div class="border-t border-gray-200 dark:border-gray-700 my-3"></div>

			<p class="text-[10px] font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500 px-3 mb-2">
				Other
			</p>
			{#each standaloneCategories as category}
				{@const isActive = activeTab === category.id}
				<button
					onclick={() => switchTab(category.id)}
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

	<!-- Right Content Area -->
	<div class="flex-1 overflow-auto p-6">
		{#if activeTab === 'models'}
			<SettingsModelsTab />
		{:else if activeTab === 'security'}
			<SettingsSecurityTab />
		{:else if activeTab === 'plugins'}
			<SettingsPluginsTab />
		{:else if activeTab === 'environment'}
			<SettingsEnvironmentTab />
		{:else if activeTab === 'interface'}
			<SettingsInterfaceTab />
		{:else if activeTab === 'files'}
			<SettingsFilesTab />
		{:else if activeTab === 'session'}
			<SettingsSessionTab />
		{:else if activeTab === 'authentication'}
			<SettingsAuthTab />
		{:else if activeTab === 'mcp-approval'}
			<SettingsMcpApprovalTab />
		{:else if activeTab === 'keybindings'}
			<SettingsKeybindingsTab />
		{:else if activeTab === 'spinner-verbs'}
			<SettingsSpinnerVerbsTab />
		{:else if activeTab === 'admin'}
			<SettingsAdminTab />
		{:else if activeTab === 'editor-sync'}
			<SettingsEditorSyncTab />
		{/if}
	</div>
</div>
