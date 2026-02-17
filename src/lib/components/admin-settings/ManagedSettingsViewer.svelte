<script lang="ts">
	import type { ManagedSettingsInfo, ClaudeSettings } from '$lib/types';
	import { MANAGED_SETTINGS_FIELDS } from '$lib/types';
	import { Lock, CheckCircle, XCircle, FileWarning, Shield } from 'lucide-svelte';

	type Props = {
		info: ManagedSettingsInfo;
	};

	let { info }: Props = $props();

	// Standard (non-managed-only) fields to display when set
	const STANDARD_FIELDS: { key: keyof ClaudeSettings; label: string }[] = [
		{ key: 'model', label: 'Model' },
		{ key: 'availableModels', label: 'Available Models' },
		{ key: 'outputStyle', label: 'Output Style' },
		{ key: 'language', label: 'Language' },
		{ key: 'alwaysThinkingEnabled', label: 'Always Thinking Enabled' },
		{ key: 'sandbox', label: 'Sandbox' },
		{ key: 'enabledPlugins', label: 'Enabled Plugins' },
		{ key: 'extraKnownMarketplaces', label: 'Extra Known Marketplaces' },
		{ key: 'env', label: 'Environment Variables' },
		{ key: 'showTurnDuration', label: 'Show Turn Duration' },
		{ key: 'spinnerTipsEnabled', label: 'Spinner Tips Enabled' },
		{ key: 'terminalProgressBarEnabled', label: 'Terminal Progress Bar' },
		{ key: 'prefersReducedMotion', label: 'Prefers Reduced Motion' },
		{ key: 'respectGitignore', label: 'Respect .gitignore' },
		{ key: 'cleanupPeriodDays', label: 'Cleanup Period (days)' },
		{ key: 'autoUpdatesChannel', label: 'Auto-Updates Channel' },
		{ key: 'teammateMode', label: 'Teammate Mode' },
		{ key: 'enableAllProjectMcpServers', label: 'Enable All Project MCP Servers' }
	];

	function hasValue(val: unknown): boolean {
		if (val === undefined || val === null) return false;
		if (Array.isArray(val)) return val.length > 0;
		if (typeof val === 'object') return Object.keys(val).length > 0;
		return true;
	}

	function formatValue(val: unknown): string {
		if (typeof val === 'boolean') return val ? 'Yes' : 'No';
		if (typeof val === 'number') return String(val);
		if (typeof val === 'string') return val;
		if (typeof val === 'object') return JSON.stringify(val, null, 2);
		return String(val);
	}

	const managedOnlyKeys = MANAGED_SETTINGS_FIELDS.map((f) => f.key);

	function getActiveStandardFields(s: ClaudeSettings) {
		return STANDARD_FIELDS.filter(
			(f) => !managedOnlyKeys.includes(f.key as (typeof managedOnlyKeys)[number]) && hasValue(s[f.key])
		);
	}
</script>

<!-- File status banner -->
<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5 mb-6">
	<div class="flex items-center gap-3">
		<Shield class="w-5 h-5 text-gray-500 dark:text-gray-400 flex-shrink-0" />
		<div class="flex-1 min-w-0">
			<h3 class="text-sm font-semibold text-gray-900 dark:text-white">Managed Settings File</h3>
			<p class="text-xs text-gray-500 dark:text-gray-400 font-mono truncate mt-0.5">
				{info.filePath}
			</p>
		</div>
		{#if info.exists}
			<span
				class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400 border border-green-200 dark:border-green-800"
			>
				<CheckCircle class="w-3.5 h-3.5" />
				Found
			</span>
		{:else}
			<span
				class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 border border-gray-200 dark:border-gray-600"
			>
				<XCircle class="w-3.5 h-3.5" />
				Not Found
			</span>
		{/if}
	</div>
</div>

{#if !info.exists}
	<!-- Empty state for non-enterprise users -->
	<div
		class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8 text-center"
	>
		<FileWarning class="w-12 h-12 text-gray-300 dark:text-gray-600 mx-auto mb-4" />
		<h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">
			No Managed Settings Found
		</h3>
		<p class="text-sm text-gray-500 dark:text-gray-400 max-w-md mx-auto">
			Managed settings are deployed by IT administrators to enforce organization-wide
			policies. They are read-only and cannot be modified by users.
		</p>
		<p class="text-xs text-gray-400 dark:text-gray-500 mt-3">
			Expected location: <code class="font-mono">{info.filePath}</code>
		</p>
	</div>
{:else if info.settings}
	{@const s = info.settings}
	{@const activeStandard = getActiveStandardFields(s)}

	<div class="space-y-6">
		<!-- Managed-only section -->
		<div
			class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5"
		>
			<div class="flex items-center gap-2 mb-4">
				<Lock class="w-4 h-4 text-amber-500" />
				<h3 class="text-base font-semibold text-gray-900 dark:text-white">
					Managed-Only Policies
				</h3>
			</div>
			<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
				These fields can only be set via the managed settings file. They enforce enterprise policies and cannot be overridden.
			</p>

			<div class="divide-y divide-gray-100 dark:divide-gray-700">
				{#each MANAGED_SETTINGS_FIELDS as field}
					{@const val = s[field.key]}
					<div class="py-3 flex items-start gap-3">
						<Lock class="w-3.5 h-3.5 text-amber-400 mt-0.5 flex-shrink-0" />
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-2">
								<span class="text-sm font-medium text-gray-900 dark:text-white">
									{field.label}
								</span>
								{#if !hasValue(val)}
									<span
										class="text-xs text-gray-400 dark:text-gray-500 italic"
									>
										Not set
									</span>
								{/if}
							</div>
							<p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
								{field.description}
							</p>
							{#if hasValue(val)}
								<div class="mt-2">
									{#if field.type === 'boolean'}
										<span
											class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium
												{val ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400' : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400'}"
										>
											{val ? 'Yes' : 'No'}
										</span>
									{:else if field.type === 'stringArray' && Array.isArray(val)}
										<div class="flex flex-wrap gap-1.5">
											{#each val as item}
												<span
													class="inline-flex items-center px-2 py-0.5 rounded-md text-xs bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400 border border-blue-200 dark:border-blue-800"
												>
													{item}
												</span>
											{/each}
										</div>
									{:else}
										<span
											class="text-sm font-mono text-gray-700 dark:text-gray-300"
										>
											{formatValue(val)}
										</span>
									{/if}
								</div>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>

		<!-- Standard settings being managed -->
		{#if activeStandard.length > 0}
			<div
				class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5"
			>
				<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">
					Managed Standard Settings
				</h3>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
					These regular settings are being enforced through the managed configuration file.
				</p>

				<div class="divide-y divide-gray-100 dark:divide-gray-700">
					{#each activeStandard as field}
						{@const val = s[field.key]}
						<div class="py-3">
							<div class="flex items-center gap-2 mb-1">
								<span class="text-sm font-medium text-gray-900 dark:text-white">
									{field.label}
								</span>
							</div>
							<div class="mt-1">
								{#if typeof val === 'boolean'}
									<span
										class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium
											{val ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400' : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400'}"
									>
										{val ? 'Yes' : 'No'}
									</span>
								{:else if Array.isArray(val)}
									<div class="flex flex-wrap gap-1.5">
										{#each val as item}
											<span
												class="inline-flex items-center px-2 py-0.5 rounded-md text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-600"
											>
												{item}
											</span>
										{/each}
									</div>
								{:else if typeof val === 'object' && val !== null}
									<pre
										class="text-xs font-mono bg-gray-50 dark:bg-gray-900 rounded p-2 overflow-x-auto text-gray-700 dark:text-gray-300"
									>{JSON.stringify(val, null, 2)}</pre>
								{:else}
									<span class="text-sm font-mono text-gray-700 dark:text-gray-300">
										{formatValue(val)}
									</span>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</div>
{/if}
