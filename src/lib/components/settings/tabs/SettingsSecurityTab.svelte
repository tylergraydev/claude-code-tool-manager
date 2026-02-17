<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { SandboxConfigEditor } from '$lib/components/sandbox';
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
		if (!s?.sandbox) return 0;
		let count = 0;
		const sb = s.sandbox;
		if (sb.enabled !== undefined) count++;
		if (sb.autoAllowBashIfSandboxed !== undefined) count++;
		if (sb.excludedCommands && sb.excludedCommands.length > 0) count++;
		if (sb.allowUnsandboxedCommands !== undefined) count++;
		if (sb.enableWeakerNestedSandbox !== undefined) count++;
		if (sb.network) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
		<SandboxConfigEditor
			{settings}
			onsave={(s) => save(s, 'Sandbox settings saved', 'Failed to save sandbox settings')}
		/>
	{/snippet}
</ScopedSettingsWrapper>
