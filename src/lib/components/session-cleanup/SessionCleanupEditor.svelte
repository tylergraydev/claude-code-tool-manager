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

	$effect(() => {
		cleanupPeriodDays =
			settings.cleanupPeriodDays !== undefined ? String(settings.cleanupPeriodDays) : '';
		autoUpdatesChannel = settings.autoUpdatesChannel ?? '';
		teammateMode = settings.teammateMode ?? '';
		plansDirectory = settings.plansDirectory ?? '';
	});

	function handleSave() {
		const days = cleanupPeriodDays.trim();
		const parsedDays = days !== '' ? parseInt(days, 10) : undefined;

		onsave({
			...settings,
			cleanupPeriodDays: parsedDays !== undefined && !isNaN(parsedDays) ? parsedDays : undefined,
			autoUpdatesChannel: autoUpdatesChannel || undefined,
			teammateMode: teammateMode || undefined,
			plansDirectory: plansDirectory.trim() || undefined
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
		</div>
	</div>

	<div class="flex justify-end">
		<button onclick={handleSave} class="btn btn-primary">
			<Save class="w-4 h-4 mr-2" />
			Save Session & Cleanup Settings
		</button>
	</div>
</div>
