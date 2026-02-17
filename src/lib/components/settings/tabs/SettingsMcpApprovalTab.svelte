<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { McpApprovalEditor } from '$lib/components/mcp-approval';
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
		if (s.enableAllProjectMcpServers !== undefined) count++;
		if (s.enabledMcpjsonServers && s.enabledMcpjsonServers.length > 0) count++;
		if (s.disabledMcpjsonServers && s.disabledMcpjsonServers.length > 0) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
		<McpApprovalEditor
			{settings}
			onsave={(s) => save(s, 'MCP approval settings saved', 'Failed to save MCP approval settings')}
		/>
	{/snippet}
</ScopedSettingsWrapper>
