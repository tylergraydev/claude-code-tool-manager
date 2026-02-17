<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { SessionCleanupEditor } from '$lib/components/session-cleanup';
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
		if (s.cleanupPeriodDays !== undefined) count++;
		if (s.autoUpdatesChannel) count++;
		if (s.teammateMode) count++;
		if (s.plansDirectory) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
		<SessionCleanupEditor
			{settings}
			onsave={(s) => save(s, 'Session & cleanup settings saved', 'Failed to save session & cleanup settings')}
		/>
	{/snippet}
</ScopedSettingsWrapper>
