<script lang="ts">
	import { ScopedSettingsWrapper } from '$lib/components/settings';
	import { ModelConfigEditor, AttributionEditor } from '$lib/components/claude-settings';
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
		if (s.model) count++;
		if (s.availableModels.length > 0) count++;
		if (s.outputStyle) count++;
		if (s.language) count++;
		if (s.alwaysThinkingEnabled !== undefined) count++;
		if (s.attributionCommit !== undefined) count++;
		if (s.attributionPr !== undefined) count++;
		return count;
	}
</script>

<ScopedSettingsWrapper {getSettingCount}>
	{#snippet children({ settings, save })}
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
	{/snippet}
</ScopedSettingsWrapper>
