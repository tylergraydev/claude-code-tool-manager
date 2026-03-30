<script lang="ts">
	import type { ClaudeSettings } from '$lib/types';
	import { AUTO_UPDATES_CHANNELS, TEAMMATE_MODES } from '$lib/types';
	import { Save } from 'lucide-svelte';

	type Props = {
		settings: ClaudeSettings;
		onsave: (settings: ClaudeSettings) => void;
	};

	let { settings, onsave }: Props = $props();

	let cleanupPeriodDays = $state<string>(
		settings.cleanupPeriodDays !== undefined ? String(settings.cleanupPeriodDays) : ''
	);
	let autoUpdatesChannel = $state(settings.autoUpdatesChannel ?? '');
	let teammateMode = $state(settings.teammateMode ?? '');
	let plansDirectory = $state(settings.plansDirectory ?? '');
	let autoMemoryEnabled = $state<boolean | undefined>(settings.autoMemoryEnabled);
	let autoMemoryDirectory = $state(settings.autoMemoryDirectory ?? '');
	let claudeMdExcludes = $state(settings.claudeMdExcludes?.join(', ') ?? '');
	let defaultAgent = $state(settings.agent ?? '');
	let disableAutoMode = $state<boolean | undefined>(settings.disableAutoMode);

	$effect(() => {
		cleanupPeriodDays =
			settings.cleanupPeriodDays !== undefined ? String(settings.cleanupPeriodDays) : '';
		autoUpdatesChannel = settings.autoUpdatesChannel ?? '';
		teammateMode = settings.teammateMode ?? '';
		plansDirectory = settings.plansDirectory ?? '';
		autoMemoryEnabled = settings.autoMemoryEnabled;
		autoMemoryDirectory = settings.autoMemoryDirectory ?? '';
		claudeMdExcludes = settings.claudeMdExcludes?.join(', ') ?? '';
		defaultAgent = settings.agent ?? '';
		disableAutoMode = settings.disableAutoMode;
	});

	function handleSave() {
		const days = cleanupPeriodDays.trim();
		const parsedDays = days !== '' ? parseInt(days, 10) : undefined;

		const excludes = claudeMdExcludes
			.split(',')
			.map((s) => s.trim())
			.filter(Boolean);

		onsave({
			...settings,
			cleanupPeriodDays: parsedDays !== undefined && !isNaN(parsedDays) ? parsedDays : undefined,
			autoUpdatesChannel: autoUpdatesChannel || undefined,
			teammateMode: teammateMode || undefined,
			plansDirectory: plansDirectory.trim() || undefined,
			autoMemoryEnabled: autoMemoryEnabled,
			autoMemoryDirectory: autoMemoryDirectory.trim() || undefined,
			claudeMdExcludes: excludes.length > 0 ? excludes : undefined,
			agent: defaultAgent.trim() || undefined,
			disableAutoMode: disableAutoMode
		});
	}
</script>

<div class="space-y-6">
	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Session & Cleanup</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Configure session cleanup, update channels, teammate mode, and plans directory
		</p>

		<div class="space-y-4">
			<div>
				<label for="cleanup-days" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Cleanup Period (days)
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Number of days before old sessions are cleaned up
				</p>
				<input
					id="cleanup-days"
					type="number"
					bind:value={cleanupPeriodDays}
					placeholder="30"
					min="1"
					class="input w-full"
				/>
			</div>

			<div>
				<label for="updates-channel" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Auto Updates Channel
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Which release channel to use for automatic updates
				</p>
				<select id="updates-channel" bind:value={autoUpdatesChannel} class="input w-full">
					{#each AUTO_UPDATES_CHANNELS as channel}
						<option value={channel.value}>{channel.label}</option>
					{/each}
				</select>
			</div>

			<div>
				<label for="teammate-mode" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Teammate Mode
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					How Claude Code runs as a background teammate
				</p>
				<select id="teammate-mode" bind:value={teammateMode} class="input w-full">
					{#each TEAMMATE_MODES as mode}
						<option value={mode.value}>{mode.label}</option>
					{/each}
				</select>
			</div>

			<div>
				<label for="plans-dir" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Plans Directory
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Directory where Claude Code stores plan files
				</p>
				<input
					id="plans-dir"
					type="text"
					bind:value={plansDirectory}
					placeholder="./plans"
					class="input w-full"
				/>
			</div>

			<div>
				<label for="default-agent" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Default Agent
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Default subagent type to use for this project
				</p>
				<input
					id="default-agent"
					type="text"
					bind:value={defaultAgent}
					placeholder="e.g. code-reviewer"
					class="input w-full"
				/>
			</div>
		</div>
	</div>

	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Memory & Instructions</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Configure auto memory and CLAUDE.md instruction loading
		</p>

		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<div>
					<label for="auto-memory" class="text-sm font-medium text-gray-700 dark:text-gray-300">
						Auto Memory
					</label>
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Automatically save and recall context between sessions
					</p>
				</div>
				<label class="relative inline-flex items-center cursor-pointer">
					<input
						id="auto-memory"
						type="checkbox"
						checked={autoMemoryEnabled === true}
						onchange={(e) => {
							const target = e.currentTarget;
							autoMemoryEnabled = target.checked ? true : undefined;
						}}
						class="sr-only peer"
					/>
					<div
						class="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500/40 rounded-full peer dark:bg-gray-600 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:after:border-gray-500 peer-checked:bg-blue-500"
					></div>
				</label>
			</div>

			<div>
				<label for="memory-dir" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					Auto Memory Directory
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Custom directory path for auto memory files
				</p>
				<input
					id="memory-dir"
					type="text"
					bind:value={autoMemoryDirectory}
					placeholder="Default: .claude/memory/"
					class="input w-full"
				/>
			</div>

			<div>
				<label for="claude-md-excludes" class="text-sm font-medium text-gray-700 dark:text-gray-300">
					CLAUDE.md Excludes
				</label>
				<p class="text-xs text-gray-500 dark:text-gray-400 mb-2">
					Comma-separated glob patterns to skip when loading CLAUDE.md files (useful for monorepos)
				</p>
				<input
					id="claude-md-excludes"
					type="text"
					bind:value={claudeMdExcludes}
					placeholder="e.g. packages/legacy/**, apps/deprecated/**"
					class="input w-full"
				/>
			</div>
		</div>
	</div>

	<div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
		<h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">Mode Restrictions</h3>
		<p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
			Control which permission modes are available
		</p>

		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<div>
					<label for="disable-auto" class="text-sm font-medium text-gray-700 dark:text-gray-300">
						Disable Auto Mode
					</label>
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Prevent use of the auto permission mode
					</p>
				</div>
				<label class="relative inline-flex items-center cursor-pointer">
					<input
						id="disable-auto"
						type="checkbox"
						checked={disableAutoMode === true}
						onchange={(e) => {
							const target = e.currentTarget;
							disableAutoMode = target.checked ? true : undefined;
						}}
						class="sr-only peer"
					/>
					<div
						class="w-9 h-5 bg-gray-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-500/40 rounded-full peer dark:bg-gray-600 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all dark:after:border-gray-500 peer-checked:bg-blue-500"
					></div>
				</label>
			</div>
		</div>
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Session Settings
		</button>
	</div>
</div>
