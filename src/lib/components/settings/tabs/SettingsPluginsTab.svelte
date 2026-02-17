<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { PluginListEditor, MarketplaceEditor } from '$lib/components/plugins';
	import { claudeSettingsLibrary } from '$lib/stores';
	import type { ClaudeSettingsScope } from '$lib/types';

	function getSettingCount(scope: ClaudeSettingsScope): number {
		if (!claudeSettingsLibrary.settings) return 0;
		const s =
			scope === 'user'
				? claudeSettingsLibrary.settings.user
				: scope === 'project'
					? claudeSettingsLibrary.settings.project
					: claudeSettingsLibrary.settings.local;
		if (!s) return 0;
		let count = 0;
		if (s.enabledPlugins && Object.keys(s.enabledPlugins).length > 0) count++;
		if (s.extraKnownMarketplaces && Object.keys(s.extraKnownMarketplaces).length > 0) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<PluginListEditor
				{settings}
				onsave={(s) => save(s, 'Plugin settings saved', 'Failed to save plugin settings')}
			/>
			<MarketplaceEditor
				{settings}
				onsave={(s) => save(s, 'Marketplace settings saved', 'Failed to save marketplace settings')}
			/>
		</div>
	{/snippet}
</ScopedSettingsWrapper>
