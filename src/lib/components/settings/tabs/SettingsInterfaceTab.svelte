<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { UITogglesEditor } from '$lib/components/ui-toggles';
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
		if (s.showTurnDuration !== undefined) count++;
		if (s.spinnerTipsEnabled !== undefined) count++;
		if (s.terminalProgressBarEnabled !== undefined) count++;
		if (s.prefersReducedMotion !== undefined) count++;
		if (s.respectGitignore !== undefined) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
		<UITogglesEditor
			{settings}
			onsave={(s) => save(s, 'UI toggle settings saved', 'Failed to save UI toggle settings')}
		/>
	{/snippet}
</ScopedSettingsWrapper>
