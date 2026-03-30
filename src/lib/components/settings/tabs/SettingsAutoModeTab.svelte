<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { AutoModeEditor } from '$lib/components/auto-mode';
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
		if (s.disableAutoMode !== undefined) count++;
		if (s.autoModeEnvironment) count++;
		if (s.autoModeAllow) count++;
		if (s.autoModeSoftDeny) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
		<AutoModeEditor
			{settings}
			onsave={(s) => save(s, 'Auto mode settings saved', 'Failed to save auto mode settings')}
		/>
	{/snippet}
</ScopedSettingsWrapper>
